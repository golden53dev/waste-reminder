# Podsumowanie Projektu — Waste Reminder (Tauri v2)

Waste Reminder to lekka, natywna aplikacja działająca w zasobniku systemowym (tray), której zadaniem jest niezawodne przypominanie o porannym wywozie odpadów komunalnych w oparciu o roczne harmonogramy JSON.

---

## 1. Zrealizowane moduły i architektura

Aplikacja została zbudowana zgodnie z założeniami: **Tauri v2 + Rust + czysty HTML/JS/CSS** (bez narzutu ciężkich frameworków frontendowych).

```
waste_reminder/
├── project_doc/              # Dokumentacja techniczna (ARCHITECTURE.md, PLAN.md, Summary.md)
└── waste-reminder/
    ├── data/                 # Przykładowy harmonogram (trash_schedule_2026.json)
    ├── scripts/              # Skrypty wybudzania systemu (macOS, Windows, Linux)
    │   ├── install-wake.sh   # Obsługa macOS (pmset) i Linux (systemd timer)
    │   └── install-wake.ps1  # Obsługa Windows (Task Scheduler WakeToRun)
    ├── src/                  # Frontend HTML/JS/CSS
    │   ├── alarm.html / js   # Okno alarmu (1 grafika, podpis frakcji, przyciski X i Opcje)
    │   ├── options.html / js # Okno konfiguracji (dźwięki, głośność, godzina, fullscreen)
    │   ├── expired.html / js # Okno wygasłego harmonogramu i importu nowego JSON
    │   ├── common.css        # Wspólne style wizualne
    │   └── assets/           # Dźwięki i grafiki pojemników (komunalne, papier, tworzywa)
    └── src-tauri/            # Backend aplikacji (Rust)
        ├── src/
        │   ├── config.rs     # Obsługa config.json (atomowy zapis, wartości domyślne)
        │   ├── schedule.rs   # Parser i walidator harmonogramów (schema_version, duplikaty dat)
        │   ├── scheduler.rs  # Logika decyzyjna (Alarm, Expired, Nothing, ostrzeżenia)
        │   └── lib.rs        # Pętla w tle, tray menu, komendy IPC, okna natywne
        ├── capabilities/     # Uprawnienia okien w Tauri v2 (core, dialog, autostart)
        ├── Cargo.toml        # Zależności i pluginy Tauri
        ├── Info.plist        # Konfiguracja macOS (LSUIElement — brak ikony w Docku)
        └── tauri.conf.json   # Konfiguracja okien, assetProtocol i budowania
```

---

## 2. Kluczowe funkcjonalności

### 🔔 Okno alarmu i powiadomień
- **Format 1 frakcji:** W oknie (520x420) wyświetlana jest dokładnie 1 główna grafika pojemnika. W przypadku zbiegu terminów kilku frakcji w jednym dniu pod spodem pojawia się elegancki podpis tekstowy (np. `Oraz: Papier`).
- **Niezależne przyciski:**
  - Przycisk **Opcje** otwiera okno konfiguracji, nie przerywając odtwarzania dźwięku ani działania alarmu.
  - Przycisk **X** zatrzymuje dźwięk, oznacza dzisiejszy dzień w `last_triggered_date` i zamyka okno alarmu.
- **Dźwięk:** Obsługuje zapętlenie audio, głośność oraz wybór pliku z poziomu ustawień za pośrednictwem protokołu zasobów Tauri (`convertFileSrc`).

### ⚙️ Okno opcji (`options.html`)
- **3 źródła dźwięku:**
  1. Domyślny dźwięk wbudowany (`alarm.mp3`).
  2. Dźwięki systemowe odczytywane automatycznie z systemu:
     - macOS: `/System/Library/Sounds/*.aiff`
     - Windows: `C:\Windows\Media\*.wav`
     - Linux: `/usr/share/sounds/**/*.ogg` / `*.wav`
  3. Własny plik dźwiękowy użytkownika wybrany przez natywne okno wyboru pliku (`tauri-plugin-dialog`).
- **Podgląd audio:** Przycisk `▶ Odtwórz` / `⏹ Zatrzymaj` pozwala przetestować wybrany dźwięk wraz z suwakiem głośności w czasie rzeczywistym.
- **Godzina alarmu i tryb pełnoekranowy:** Możliwość dostosowania godziny powiadomienia (HH:MM) oraz włączenia trybu fullscreen.

### 🛡️ Zasobnik systemowy (Tray) i Autostart
- **Jedna instancja (`single-instance`):** Próba uruchomienia kolejnej kopii programu kończy się cichym zamknięciem duplikatu.
- **Autostart:** Plugin `tauri-plugin-autostart` włączony domyślnie, z możliwością przełączenia w menu zasobnika.
- **Pozycje w menu tray:**
  - *Następny wywóz: [Data] ([Frakcje])* (informacyjna, wyszarzona).
  - *Włączony* (checkbox sterujący aktywnością alarmu).
  - *Uruchamiaj z systemem* (checkbox autostartu).
  - *Testuj alarm* (natychmiastowe wywołanie okna alarmu do testów).
  - *Opcje...* (otwiera okno opcji).
  - *Wczytaj nowy JSON...* (otwiera kreator importu).
  - *Zakończ* (zamyka aplikację).
- **Ostrzeżenie o końcu harmonogramu:** Gdy do ostatniej daty w harmonogramie zostaje `≤ 7 dni`, tooltip tray informuje o wygasaniu: `Harmonogram kończy się za X dni`.

### 📅 Zarządzanie wygaśnięciem harmonogramu (`expired.html`)
- W przypadku zakończenia ważności harmonogramu aplikacja raz dziennie wyświetla dedykowane okno z informacją i przyciskiem wczytania nowego pliku.
- Zaimplementowano bezpieczny import: plik JSON jest walidowany pod kątem zgodności schematu, braku duplikatów i poprawności roku, po czym atomowo zapisywany w katalogu konfiguracyjnym aplikacji jako `trash_schedule_YYYY.json`.

### ⏰ Skrypty wybudzania systemu (`scripts/`)
Ponieważ żadna aplikacja desktopowa nie potrafi samodzielnie włączyć wyłączonego komputera, przygotowano skrypty integrujące się z mechanizmami systemu operacyjnego:
- **macOS (`scripts/install-wake.sh`):** Korzysta z `sudo pmset schedule wakeorpoweron` i planuje wybudzenie komputera 5 minut przed godziną alarmu dla maks. 20 najbliższych dat.
- **Windows (`scripts/install-wake.ps1`):** Konfiguruje zadanie w Harmonogramie Zadań (Task Scheduler) z flagą `WakeToRun`.
- **Linux (`scripts/install-wake.sh`):** Konfiguruje timer systemd użytkownika z flagą `WakeSystem=true`.
> [!IMPORTANT]
> Wybudzanie działa **ze stanu uśpienia** (sleep / standby / suspend / S3). Komputer nie wybudzi się z pełnego wyłączenia zasilania (shutdown).

### 🚀 CI/CD i dystrybucja
- Utworzono GitHub Actions workflow (`.github/workflows/build.yml`) budujący aplikację w macierzy: `macos-latest` (uniwersalny binary ARM64 + x86_64), `windows-latest` (`.msi`, `.exe`) oraz `ubuntu-22.04` (`.AppImage`, `.deb`).

---

## 3. Testy automatyczne

Kod Rust posiada pełen zestaw testów jednostkowych (`cargo test`), które weryfikują:
- Poprawność wczytywania i uzupełniania brakujących pól w `config.json`.
- Atomowy zapis i odporność na uszkodzone pliki konfiguracyjne.
- Walidację wersji schematu, wykrywanie duplikatów i dat spoza zadeklarowanego roku.
- Wyznaczanie frakcji na dany dzień (dla 0, 1 lub 2 frakcji równocześnie).
- Wyznaczanie najbliższego wywozu i daty końcowej harmonogramu.
- Decyzje schedulera: alarm przed czasem, w czasie wywozu, po oknie (catch-up), po wygaszeniu oraz w okresie ostrzegania.

**Wynik testów:** `20 passed; 0 failed`.

---

## 4. Jak najwygodniej uruchomić i zainstalować aplikację?

Najwygodniejszym podejściem dla użytkownika końcowego jest **pobranie gotowej paczki instalacyjnej z GitHub Releases** (tworzonej automatycznie przez GitHub Actions) wraz ze skryptem wybudzającym, zamiast ręcznego kompilowania środowiska Node.js i Rust.

---

### Sposób A: Pobranie gotowej aplikacji z GitHuba (Zalecany)

1. Przejdź do zakładki **Releases** w swoim repozytorium na GitHubie.
2. Pobierz odpowiednią paczkę dla swojego systemu operacyjnego:
   - **macOS:** plik `.dmg` lub `.app`
   - **Windows:** instalator `.msi` lub `.exe`
   - **Linux:** paczka `.AppImage` lub `.deb`
3. Pobierz folder `scripts/` oraz plik z harmonogramem `data/trash_schedule_2026.json`.

#### 🍏 macOS (Krok po kroku)
1. Otwórz pobrany plik `.dmg` i przeciągnij `waste-reminder.app` do folderu `Applications` (`Programy`).
2. **Pierwsze uruchomienie (Gatekeeper):**
   - Ponieważ aplikacja nie posiada płatnego certyfikatu Apple Developer, kliknij na nią **prawym przyciskiem myszy** (lub przytrzymaj `Control` i kliknij) w folderze `Applications`.
   - Wybierz z menu kontekstowego **Otwórz** (Open), a w wyskakującym okienku ponownie kliknij **Otwórz**.
3. **Harmonogram odpadów:**
   - Skopiuj plik `trash_schedule_2026.json` do katalogu:
     `~/Library/Application Support/com.app.waste-reminder/`
     *(Katalog ten tworzy się automatycznie po pierwszym uruchomieniu aplikacji. Możesz też po prostu kliknąć ikonę w zasobniku i wybrać „Wczytaj nowy JSON...”)*.
4. **Wybudzanie ze snu:**
   - W terminalu przejdź do folderu ze skryptem i uruchom:
     ```bash
     chmod +x scripts/install-wake.sh
     ./scripts/install-wake.sh
     ```
   - Skrypt poprosi o hasło `sudo` i ustawi wybudzanie w macOS `pmset`.

---

#### 🪟 Windows (Krok po kroku)
1. Uruchom pobrany instalator `.msi` lub `.exe` i przejdź przez standardowy proces instalacji.
2. Uruchom program *Waste Reminder* (pojawi się ikona w zasobniku systemowym obok zegara – może być ukryta pod strzałką `^`).
3. **Harmonogram odpadów:**
   - Skopiuj `trash_schedule_2026.json` do:
     `%APPDATA%\com.app.waste-reminder\` (`C:\Users\<Twój_Użytkownik>\AppData\Roaming\com.app.waste-reminder\`)
     *albo skorzystaj z opcji „Wczytaj nowy JSON...” w menu ikony tray*.
4. **Wybudzanie ze snu:**
   - Otwórz **PowerShell jako Administrator** (kliknij Start prawym przyciskiem -> *Terminal (Administrator)* lub *Windows PowerShell (Administrator)*).
   - Przejdź do folderu ze skryptem i uruchom:
     ```powershell
     Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
     .\scripts\install-wake.ps1
     ```
   - Skrypt utworzy zadanie w Harmonogramie Zadań z uprawnieniem wybudzania komputera.

---

#### 🐧 Linux (Krok po kroku)
1. W przypadku paczki `.deb`:
   ```bash
   sudo apt install ./waste-reminder_0.1.0_amd64.deb
   ```
   W przypadku `.AppImage`:
   ```bash
   chmod +x waste-reminder_0.1.0_amd64.AppImage
   ./waste-reminder_0.1.0_amd64.AppImage
   ```
   *(Wymagane pakiety systemowe: `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`).*
2. **Harmonogram odpadów:**
   - Umieść plik JSON w katalogu:
     `~/.config/com.app.waste-reminder/trash_schedule_2026.json`
3. **Wybudzanie ze snu:**
   - Uruchom skrypt konfigurujący timer systemd:
     ```bash
     chmod +x scripts/install-wake.sh
     ./scripts/install-wake.sh
     ```

---

### Sposób B: Uruchomienie ze źródeł (Tryb Developerski)

Wymagania: zainstalowany **Node.js** (v18+) oraz środowisko **Rust / Cargo** (1.85+).

```bash
# 1. Wejdź do katalogu projektu
cd waste-reminder

# 2. Zainstaluj zależności JS
npm install

# 3. Uruchomienie w trybie deweloperskim na żywo
npm run tauri dev

# 4. Zbudowanie gotowej paczki produkcyjnej dla bieżącego systemu
npm run tauri build
```
Gotowe binaria pojawią się w folderze `waste-reminder/src-tauri/target/release/bundle/`.
