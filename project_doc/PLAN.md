# PLAN DZIAŁANIA — Waste Reminder (Tauri v2)

Cel: lekka appka tray, która rano w dzień wywozu odpala fullscreen z grafiką + dźwiękiem (przycisk `X` i `Opcje`)
i pilnuje, czy harmonogram `schedule.json` oibecnie trash_schedule.json nie wygasł.
Platformy: **Linux + Windows** (macOS opcjonalnie, później).

## Założenia
- Tauri **v2** (plugin autostart 2.5.x, tray wbudowany w core, plugin notification opcjonalnie).
- Frontend: czysty HTML/CSS/JS (bez frameworka) — Vite opcjonalnie.
- Dane: `trash_schedule.json`  (read-only, wymienialny) + `config.json` (ustawienia usera, osobno!).
- Aplikacja **nie budzi komputera sama** — robi to system (Task Scheduler / systemd / pmset). Patrz ARCHITECTURE.md.

## Etap 0 — Dane (przed kodem)
- [ ] Zweryfikować `trash_schedule.json` ze zdjęciem (dane z OCR/AI mogą mieć błędy w datach).
- [ ] Ustalić znaczenie `*` (co 4 tyg.) — dla Twojej nieruchomości: czy to w ogóle dotyczy? usuniete poprosu daty sa co 4 tygodnie i sie chyba
- [ ] Walidacja: daty w formacie `YYYY-MM-DD`, posortowane, brak duplikatów, rok zgodny z `year`.
- [ ] Dodać `schema_version` do JSON.

## Etap 1 — Szkielet
- [ ] `npm create tauri-app@latest` (vanilla / JS) → uruchamia się `tauri dev` na Linux i Win.
- [ ] Przygotować `assets/`: `komunalne.png`, `tworzywa.png`, `papier.png`, `expired.png`, `alarm.wav`.
- [ ] Repo git + `.gitignore` (target/, node_modules/).

## Etap 2 — Rdzeń (Rust)
- [ ] Parser `trash_schedule.json` + `config.json` (serde), błędy → czytelny komunikat.
- [ ] Funkcja `fractions_for(date) -> Vec<Fraction>` (obsługa 2 frakcji tego samego dnia).
- [ ] Funkcja `next_pickup(date)` i `last_date()`.
- [ ] Testy jednostkowe (`cargo test`): dzień z jedną frakcją, z dwiema, bez, po końcu roku, zmiana roku.
- [ ] Stan `last_triggered_date` zapisywany na dysk (przeżywa restart).

- [ ] Ikona + menu: status (aktywny / wyłączony / wygasły), następny wywóz, Włącz/Wyłącz, Opcje, Testuj alarm, Zakończ.
- [ ] Zmiana ikony przy ostrzeżeniu (harmonogram kończy się).


## Etap 3 — Tray
- [ ] Autostart (`tauri-plugin-autostart`).
- [ ] Single instance (`tauri-plugin-single-instance`) — nie dwie kopie.

## Etap 4 — Scheduler
- [ ] Pętla w tle (tokio) co 30 s: czy `now` w oknie `check_window` i `today != last_triggered_date`.
Optymalizacja: zgoda. Zapisuję na później (Etap 4b), główny plan idzie dalej:
- liczyć czas do najbliższego alarmu i spać do niego (zamiast pętli co 30 s)
- po wybudzeniu z uśpienia wykryć skok zegara i sprawdzić od razu
- 30 s obciąża CPU znikomo, więc teraz to nie problem
- [ ] **Catch-up**: przy starcie apki, jeśli jest dziś wywóz i okno minęło (np. PC włączony o 8:00) → pokaż alarm (konfigurowalne).
- [ ] Zapis `last_triggered_date` dopiero po kliknięciu `X`.

## Etap 5 — Okno alarmu
moze zostac jak jest mniejsze i moze byc 1 frakcja wyswietlana
- wazne jest okieno opcji do zmiany dzwienku / systemowego / userowego
- [ ] Okno fullscreen, always-on-top, bez ramek, tworzone na żądanie (nie ciągle).
- [ ] Grafika frakcji (1 lub 2), dźwięk w pętli (HTML Audio), przycisk `X`, przycisk `Opcje`.
- [ ] Opcje: wybór pliku dźwięku (dialog plugin), godzina, wł./wył. fullscreen, głośność.
- [ ] Multi-monitor: okno na monitorze głównym (lub wszystkich — opcja).

## Etap 6 — Wygaśnięcie harmonogramu
- [ ] Ostrzeżenie gdy do `last_date` ≤ `warn_days_before_end`.
- [ ] Po `last_date`: rano ekran „Harmonogram wygasł” + przycisk „Wczytaj nowy JSON” (file dialog → walidacja → podmiana).

## Etap 7 — Wybudzanie systemu (poza appką)
na mac najwaznijsza

- [ ] **Windows**: zadanie Task Scheduler z `WakeToRun` (skrypt PowerShell `install-wake.ps1`).
- [ ] **Linux**: `systemd` timer z `WakeSystem=true` (user/system unit) albo `rtcwake`.
- [ ] (macOS: `pmset schedule wakeorpoweron`.)
- [ ] Skrypt generuje wpisy z dat w `trash_schedule.json`.

## Etap 8 — Build i dystrybucja
na mac najwaznijsza
- [ ] `tauri build`: Windows `.msi/.exe`, Linux `.AppImage/.deb`.
- [ ] Test na czystej maszynie/VM (WebView2 na Win, `webkit2gtk` na Linux).
- [ ] Opcjonalnie GitHub Actions (matrix 3 OS) + updater.

## Kolejność priorytetów (MVP → reszta)
1. Etapy 0–2 (dane + logika + testy)
2. Etap 4 + 5 (alarm działa)
3. Etap 3 (tray + autostart)
4. Etap 6 (wygaśnięcie)
5. Etap 7 (wybudzanie)
6. Etap 8 (paczki)

## Ryzyka
| Ryzyko | Mitygacja |
|---|---|
| Błędne daty z OCR | Ręczna weryfikacja + walidator + test „wszystkie daty to dni robocze” |
| PC wyłączony/uśpiony o 6:30 | Wake z systemu + catch-up przy starcie |
| Autoplay audio zablokowane w WebView | Dźwięk po stronie Rust (`rodio`) jako plan B |
| Wayland: always-on-top / fullscreen bywa ignorowane | Testy na docelowym kompozytorze; fallback: maximize + notyfikacja |
| Tray na GNOME bez AppIndicator | Wymagane rozszerzenie; udokumentować |
