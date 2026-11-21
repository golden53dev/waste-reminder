# Waste Reminder

Aplikacja przypominająca o terminach wywozu odpadów komunalnych dla systemu macOS, Windows i Linux. Aplikacja działa dyskretnie w zasobniku systemowym (tray), powiadamia o zbiórkach odpadów w dniu wywozu i pozwala zaplanować wybudzanie komputera.

---

## 1. Instalacja i dane harmonogramu

Aplikacja przechowuje konfigurację (`config.json`) oraz harmonogramy (`trash_schedule_YYYY.json`) w dedykowanym katalogu konfiguracyjnym systemu:

- **macOS:** `~/Library/Application Support/com.app.waste-reminder/`
- **Windows:** `%APPDATA%\com.app.waste-reminder\` (`C:\Users\<Użytkownik>\AppData\Roaming\com.app.waste-reminder\`)
- **Linux:** `~/.config/com.app.waste-reminder/`

### Kopiowanie harmonogramu
Przykładowy plik z danymi (`data/trash_schedule_2026.json`) należy skopiować do powyższego katalogu lub zaimportować bezpośrednio w aplikacji przez opcję w menu tray **„Wczytaj nowy JSON...”**.

---

## 2. macOS – Uruchomienie aplikacji niepodpisanej (Gatekeeper)

Przy pierwszym uruchomieniu pobranej paczki `.app` / `.dmg` system macOS może wyświetlić komunikat o nieznanym deweloperze:
1. Kliknij na plik aplikacji **prawym przyciskiem myszy** (lub przytrzymaj klawisz `Control` i kliknij).
2. Z menu kontekstowego wybierz **Otwórz** (Open).
3. W oknie dialogowym kliknij **Otwórz** (Open).
Kolejne uruchomienia będą odbywać się standardowo.

---

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
