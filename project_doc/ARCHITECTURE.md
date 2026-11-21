# ARCHITEKTURA — Waste Reminder (Tauri v2)

## 1. Widok ogólny
```
┌──────────────── System (OS) ────────────────┐
│ Wake: Task Scheduler / systemd / pmset      │  ← budzi komputer przed 06:30
└───────────────┬─────────────────────────────┘
                │ (autostart / już działa)
┌───────────────▼─────────────────────────────┐
│ Tauri app (Rust core)                       │
│  ├─ schedule  (parser, walidacja, zapytania)│
│  ├─ config    (ustawienia, last_triggered)  │
│  ├─ scheduler (pętla 30 s, catch-up)        │
│  ├─ tray      (menu, status, ikony)         │
│  ├─ windows   (alarm, opcje, expired)       │
│  └─ commands  (IPC dla frontendu)           │
└───────────────┬─────────────────────────────┘
                │ IPC (invoke / events)
┌───────────────▼─────────────────────────────┐
│ Frontend (HTML/CSS/JS, WebView)             │
│  alarm.html · options.html · expired.html   │
└─────────────────────────────────────────────┘
```

## 2. Struktura projektu
```
waste-reminder/
├── src/                      # frontend
│   ├── alarm.html / alarm.js
│   ├── options.html / options.js
│   ├── expired.html / expired.js
│   ├── common.css
│   └── assets/               # komunalne.png, tworzywa.png, papier.png, expired.png, alarm.wav
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs            # setup, pluginy, rejestracja komend
│   │   ├── schedule.rs       # modele + parser + walidacja + zapytania
│   │   ├── config.rs         # Config, load/save, last_triggered
│   │   ├── scheduler.rs      # pętla czasowa, catch-up
│   │   ├── tray.rs           # menu + status
│   │   ├── windows.rs        # tworzenie okien
│   │   └── commands.rs       # #[tauri::command]
│   ├── capabilities/default.json
│   ├── tauri.conf.json
│   └── Cargo.toml
├── scripts/
│   ├── install-wake.ps1      # Windows: Task Scheduler WakeToRun
│   └── install-wake.sh       # Linux: systemd timer WakeSystem=true
├── data/schedule.json        # przykład / domyślny
├── PLAN.md
└── ARCHITECTURE.md
```

## 3. Pluginy Tauri
| Plugin | Po co |
|---|---|
| tray (core, feature `tray-icon`) | ikona + menu |
| `tauri-plugin-autostart` (2.5.x) | start z systemem |
| `tauri-plugin-single-instance` | jedna kopia |
| `tauri-plugin-dialog` | wybór pliku dźwięku / JSON |
| `tauri-plugin-fs` / własne komendy | odczyt/zapis plików poza sandboxem |
| `tauri-plugin-notification` (opcja) | fallback gdy fullscreen niedostępny |

Uwaga: capabilities (uprawnienia) w Tauri v2 trzeba jawnie nadać per okno.

## 4. Dane

### schedule.json (read-only, wymienialny)
```json
{
  "schema_version": 1,
  "region": "Olesno II",
  "year": 2026,
  "fractions": {
    "komunalne": { "label": "Zmieszane", "image": "komunalne.png", "dates": ["2026-01-13"] },
    "tworzywa":  { "label": "Plastik",   "image": "tworzywa.png",  "dates": ["2026-01-20"] },
    "papier":    { "label": "Papier",    "image": "papier.png",    "dates": ["2026-01-07"] }
  }
}
```
- Usunąć `settings` z tego pliku → ustawienia do `config.json` (zmiana harmonogramu nie nadpisuje ustawień usera).
- Pole `every_4_weeks_dates` zostaje tylko jeśli faktycznie potrzebne (filtr); inaczej wyrzucić.
- Czas liczony w strefie **Europe/Warsaw** (lokalny czas systemu, bez UTC).

### config.json (`app_config_dir`)
```json
{
  "enabled": true,
  "check_window": ["06:00", "07:00"],
  "alert_time": "06:30",
  "sound_file": null,
  "volume": 0.8,
  "fullscreen": true,
  "catch_up_on_start": true,
  "warn_days_before_end": 7,
  "last_triggered_date": null,
  "schedule_path": null
}
```
Lokalizacje: Windows `%APPDATA%\waste-reminder\`, Linux `~/.config/waste-reminder/`.

## 5. Moduły Rust (odpowiedzialności)
- **schedule.rs**: `load()`, `validate()`, `fractions_for(NaiveDate)`, `next_pickup(NaiveDate)`, `last_date()`. Czyste funkcje → łatwe testy. Crate: `chrono`, `serde`, `serde_json`.
- **config.rs**: load/save atomowy (zapis do pliku tymczasowego + rename).
- **scheduler.rs**: `tokio::time::interval(30s)`; decyzja `should_fire(now, config, schedule)`; przy starcie wywołanie catch-up.
- **tray.rs**: budowa menu, odświeżanie tekstu statusu po zmianie stanu.
- **windows.rs**: `open_alarm(fractions)`, `open_options()`, `open_expired()`; alarm jako okno tworzone w locie (`WebviewWindowBuilder`).
- **commands.rs**: `get_today_fractions`, `get_config`, `save_config`, `ack_alarm` (zapis `last_triggered_date`), `pick_sound`, `import_schedule`.

## 6. Maszyna stanów
```
         ┌────────────┐ enabled=false ┌──────────┐
 start → │   IDLE     │──────────────▶│ DISABLED │
         └─────┬──────┘               └──────────┘
               │ tick: dziś wywóz ∧ okno ∧ !triggered
               ▼
         ┌────────────┐  klik X   ┌──────────────┐
         │  ALARM     │──────────▶│ ACKED (dzień)│→ IDLE następnego dnia
         └────────────┘           └──────────────┘
 tick: dziś > last_date  → EXPIRED (ekran „zaktualizuj JSON”)
 tick: last_date − dziś ≤ warn → IDLE + flaga WARNING (ikona tray)
```

## 7. Przepływ alarmu
1. `scheduler` → `should_fire == true`.
2. `windows::open_alarm(frakcje)` → okno fullscreen / always-on-top.
3. Frontend pobiera `get_today_fractions`, pokazuje grafikę(i), zapętla dźwięk.
4. `X` → `ack_alarm` (zapis `last_triggered_date = today`), zamknięcie okna, stop dźwięku.
5. `Opcje` → okno ustawień (nie zamyka alarmu; dźwięk można wyciszyć).

## 8. Wybudzanie komputera (WAŻNE)
Appka w tle **nie obudzi** uśpionego/wyłączonego komputera. Potrzebny mechanizm systemowy:

| OS | Mechanizm | Uwagi |
|---|---|---|
| Windows | Task Scheduler, `WakeToRun`, trigger o 06:25 w dni wywozu | Działa z uśpienia/hibernacji, nie z pełnego shutdown (poza BIOS RTC wake). Wymaga zezwolenia na „wake timers” w planie zasilania. |
| Linux | `systemd` timer z `WakeSystem=true` lub `rtcwake -m mem -t ...` | Uśpienie (suspend) OK; shutdown wymaga ustawienia w BIOS/UEFI. |
| macOS | `sudo pmset schedule wakeorpoweron "MM/dd/yy HH:mm:ss"` | Opcjonalnie. |

Skrypty `install-wake.*` czytają `schedule.json` i tworzą wpisy na najbliższe daty; uruchamiane ręcznie lub z opcji w tray („Zaplanuj wybudzanie”).

## 9. Ważne uwagi / pułapki
- **Dźwięk**: autoplay w WebView może być blokowany → plan B: `rodio` w Rust.
- **Wayland**: `always_on_top` i fullscreen mogą być ograniczone przez kompozytor; sprawdzić na Hyprland/GNOME.
- **Tray na Linuxie**: GNOME wymaga rozszerzenia AppIndicator; KDE/Hyprland (waybar tray) działa.
- **Windows**: WebView2 (zwykle już jest w Win10/11); instalator `.msi`/NSIS.
- **Single instance**: bez tego autostart + ręczny start = dwa alarmy.
- **Dzień z 2 frakcjami**: np. 2026-... sprawdzić czy kolizje istnieją w danych; UI ma pokazywać obie.
- **Zmiana roku**: `last_date` = ostatnia data w pliku, nie 31 grudnia. Ostrzeżenie np. 7 dni przed.
- **Czas systemowy / DST**: używać `chrono::Local`; test na dniach zmiany czasu (marzec/październik).
- **Bezpieczeństwo**: nie nadawać szerokich uprawnień fs; wczytywany JSON walidować (rozmiar, schemat) przed podmianą.
- **Logowanie**: `tauri-plugin-log` do pliku — pomaga debugować „czemu nie zadzwoniło rano”.
- **Test alarmu**: pozycja w tray „Testuj alarm” ignoruje `last_triggered_date`.
- **Rozszerzenia później**: przypomnienie dzień wcześniej wieczorem (wystawienie pojemników), powiadomienie mobilne, import z iCal/CSV gminy.
