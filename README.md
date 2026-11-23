# Waste Reminder

Aplikacja przypominająca o terminach wywozu odpadów komunalnych dla systemu macOS, Windows i Linux. Aplikacja działa dyskretnie w zasobniku systemowym (tray), powiadamia o zbiórkach odpadów w dniu wywozu i pozwala zaplanować wybudzanie komputera.

https://github.com/golden53dev/waste-reminder/releases/
---

## 1. Instalacja i dane harmonogramu

Aplikacja przechowuje konfigurację (`config.json`) oraz harmonogramy (`trash_schedule_YYYY.json`) w dedykowanym katalogu konfiguracyjnym systemu:

- **macOS:** `~/Library/Application Support/com.app.waste-reminder/`
- **Windows:** `%APPDATA%\com.app.waste-reminder\` (`C:\Users\<Użytkownik>\AppData\Roaming\com.app.waste-reminder\`)
- **Linux:** `~/.config/com.app.waste-reminder/`

### Kopiowanie harmonogramu
Przykładowy plik z danymi (`data/trash_schedule_2026.json`) należy skopiować do powyższego katalogu lub zaimportować bezpośrednio w aplikacji przez opcję w menu tray **„Wczytaj nowy JSON...”**.

---

## 2. macOS — pobranie i instalacja wydania

Wydania aplikacji: [GitHub Releases](https://github.com/golden53dev/waste-reminder/releases). Dla Maca pobierz plik `.dmg` z najnowszego wydania (np. `waste-reminder_0.1.1_universal.dmg`). Wersja universal działa na Macach z procesorem Apple Silicon oraz Intel.

1. Otwórz stronę [Releases](https://github.com/golden53dev/waste-reminder/releases) i wybierz najnowsze wydanie `Waste Reminder v0.1.1`.
2. W sekcji **Assets** pobierz plik `.dmg`. Jeśli wydanie jest oznaczone jako **Draft**, musi je najpierw opublikować właściciel repozytorium przyciskiem **Publish release**.
3. Otwórz pobrany obraz `.dmg`. W oknie instalatora przeciągnij `waste-reminder.app` do folderu **Applications** (`Programy`).
4. Wysuń obraz instalatora, a następnie uruchom **Waste Reminder** z folderu **Applications**.
5. Aplikacja działa w pasku menu macOS; nie pokazuje ikony w Docku. Kliknij jej ikonę w pasku menu, aby otworzyć opcje, przetestować alarm albo zaimportować harmonogram.

### Pierwsze uruchomienie macOS

Wydanie nie jest podpisane certyfikatem Apple Developer, więc Gatekeeper może zablokować pierwsze uruchomienie. W Finderze otwórz folder **Applications**, kliknij aplikację prawym przyciskiem myszy (lub `Control` + kliknięcie), wybierz **Open / Otwórz**, a następnie potwierdź **Open / Otwórz** w oknie systemowym. Jeśli tej opcji nie ma, spróbuj uruchomić aplikację raz, a potem wybierz **System Settings → Privacy & Security → Open Anyway**.

### Harmonogram na Macu

Przy pierwszym uruchomieniu aplikacja kopiuje do swojego katalogu konfiguracji przykładowy harmonogram `2026` dla regionu **Olesno II**, jeśli nie ma tam jeszcze żadnego harmonogramu. Jeżeli mieszkasz w innym regionie, kliknij ikonę Waste Reminder w pasku menu i wybierz **Wczytaj nowy JSON...**, a następnie wskaż swój plik harmonogramu. Plik musi być zgodny ze schematem aplikacji. Możesz też umieścić go ręcznie w `~/Library/Application Support/com.app.waste-reminder/` pod nazwą `trash_schedule_RRRR.json`, np. `trash_schedule_2026.json`.

Wbudowany alarm `alarm.mp3` jest używany automatycznie. Nie trzeba pobierać go osobno.

## 3. Skrypty wybudzania systemu (scripts/)

Aby komputer wybudził się samoczynnie przed porannym alarmem (domyślnie 5 minut przed godziną powiadomienia, np. o 06:25), przygotowano dedykowane skrypty planujące wybudzanie na 20 najbliższych terminów wywozu.

> [!IMPORTANT]
> **Uwaga:** Funkcja wybudzania (`wake`) działa **wyłącznie ze stanu uśpienia** (sleep / standby / suspend). Wybudzanie **nie działa** przy całkowicie wyłączonym zasilaniu komputera (full power off / shutdown).

### macOS
Skrypt konfiguruje harmonogram systemowy za pomocą narzędzia `pmset`:
```bash
chmod +x scripts/install-wake.sh
./scripts/install-wake.sh
```
*(Wymaga uprawnień administratora – poprosi o hasło `sudo` w celu dodania wpisów `pmset schedule wakeorpoweron`).*

### Windows
Skrypt tworzy zadanie w Harmonogramie Zadań systemu Windows z flagą `WakeToRun`:
```powershell
# Uruchom PowerShell jako Administrator
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
.\scripts\install-wake.ps1
```

### Linux
Skrypt tworzy i aktywuje timer systemd dla użytkownika (`waste-reminder-wake.timer`) z parametrem `WakeSystem=true`:
```bash
chmod +x scripts/install-wake.sh
./scripts/install-wake.sh
```

---

## 4. Budowanie projektu

Wymagania: Node.js (v18+) oraz Rust (1.85+).

```bash
# Instalacja zależności
npm install

# Uruchomienie w trybie developerskim
npm run tauri dev

# Budowanie paczki produkcyjnej
npm run tauri build
```
