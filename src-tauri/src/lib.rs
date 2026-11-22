use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::DialogExt;
mod config;
mod schedule;
mod scheduler;

const DEFAULT_SCHEDULE: &str = include_str!("../../data/trash_schedule_2026.json");

fn install_default_schedule_if_missing(dir: &std::path::Path) -> Result<(), std::io::Error> {
    let has_schedule = std::fs::read_dir(dir)?.filter_map(Result::ok).any(|entry| {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        name.starts_with("trash_schedule_") && name.ends_with(".json")
    });

    if !has_schedule {
        let schedule = schedule::Schedule::parse(DEFAULT_SCHEDULE)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let path = dir.join(format!("trash_schedule_{}.json", schedule.year));
        std::fs::write(path, DEFAULT_SCHEDULE)?;
    }
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct FractionItem {
    pub key: String,
    pub label: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SoundItem {
    pub name: String,
    pub path: String,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct ImportResult {
    pub success: bool,
    pub message: String,
    pub year: Option<i32>,
}

fn open_expired_window(handle: &tauri::AppHandle) {
    if handle.get_webview_window("expired").is_none() {
        let _ = WebviewWindowBuilder::new(
            handle,
            "expired",
            WebviewUrl::App("expired.html".into()),
        )
        .title("Harmonogram wygasł")
        .inner_size(440.0, 320.0)
        .resizable(false)
        .center()
        .build();
    }
}

fn open_alarm_window(handle: &tauri::AppHandle, fullscreen: bool) {
    if handle.get_webview_window("alarm").is_none() {
        let mut b = WebviewWindowBuilder::new(
            handle,
            "alarm",
            WebviewUrl::App("alarm.html".into()),
        )
        .always_on_top(true)
        .decorations(false)
        .transparent(true);
        b = if fullscreen {
            b.fullscreen(true)
        } else {
            b.inner_size(520.0, 420.0).resizable(false).center()
        };
        let _ = b.build();
    }
}

#[tauri::command]
fn open_options(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("options") {
        let _ = w.show();
        let _ = w.set_focus();
    } else {
        WebviewWindowBuilder::new(
            &app,
            "options",
            WebviewUrl::App("options.html".into()),
        )
        .title("Waste Reminder - Opcje")
        .inner_size(480.0, 560.0)
        .resizable(false)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_config(app: tauri::AppHandle) -> Result<config::Config, String> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("config.json");
    config::Config::load(&path)
}

#[tauri::command]
fn save_config(app: tauri::AppHandle, updated: config::Config) -> Result<(), String> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("config.json");
    let mut current = config::Config::load(&path).unwrap_or_default();
    current.sound_file = updated.sound_file;
    current.volume = updated.volume;
    current.alert_time = updated.alert_time;
    current.fullscreen = updated.fullscreen;
    current.save(&path)?;
    Ok(())
}

#[tauri::command]
fn get_available_sounds() -> Vec<SoundItem> {
    let mut list = vec![SoundItem {
        name: "Domyślny (alarm.mp3)".to_string(),
        path: "".to_string(),
    }];

    #[cfg(target_os = "windows")]
    {
        let win_media = std::path::Path::new(r"C:\Windows\Media");
        if let Ok(entries) = std::fs::read_dir(win_media) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(ext) = p.extension() {
                    if ext.to_string_lossy().to_lowercase() == "wav" {
                        let name = p
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        list.push(SoundItem {
                            name: format!("Windows: {}", name),
                            path: p.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let mac_sounds = std::path::Path::new("/System/Library/Sounds");
        if let Ok(entries) = std::fs::read_dir(mac_sounds) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(ext) = p.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ext_str == "aiff" || ext_str == "wav" {
                        let name = p
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        list.push(SoundItem {
                            name: format!("macOS: {}", name),
                            path: p.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        fn scan_dir(dir: &std::path::Path, list: &mut Vec<SoundItem>, depth: u32) {
            if depth > 2 {
                return;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        scan_dir(&p, list, depth + 1);
                    } else if let Some(ext) = p.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if ext_str == "ogg" || ext_str == "oga" || ext_str == "wav" {
                            let name = p
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            list.push(SoundItem {
                                name: format!("Linux: {}", name),
                                path: p.to_string_lossy().to_string(),
                            });
                        }
                    }
                }
            }
        }
        scan_dir(std::path::Path::new("/usr/share/sounds"), &mut list, 0);
    }

    list
}

#[tauri::command]
async fn pick_sound_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let res = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("Dźwięki", &["mp3", "wav", "ogg", "aiff", "flac"])
            .blocking_pick_file()
            .map(|p| p.to_string())
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(res)
}

#[tauri::command]
fn close_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_schedule_dialog(app: tauri::AppHandle) -> Result<ImportResult, String> {
    let picked = tauri::async_runtime::spawn_blocking({
        let app = app.clone();
        move || {
            app.dialog()
                .file()
                .add_filter("JSON Harmonogramu", &["json"])
                .blocking_pick_file()
        }
    })
    .await
    .map_err(|e| e.to_string())?;

    let path = match picked {
        Some(p) => match p {
            tauri_plugin_dialog::FilePath::Path(buf) => buf,
            tauri_plugin_dialog::FilePath::Url(url) => match url.to_file_path() {
                Ok(p) => p,
                Err(_) => {
                    return Ok(ImportResult {
                        success: false,
                        message: "Niepoprawna ścieżka pliku".to_string(),
                        year: None,
                    });
                }
            },
        },
        None => {
            return Ok(ImportResult {
                success: false,
                message: "Anulowano wybór pliku".to_string(),
                year: None,
            });
        }
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            return Ok(ImportResult {
                success: false,
                message: format!("Błąd odczytu pliku: {e}"),
                year: None,
            });
        }
    };

    let parsed = match schedule::Schedule::parse(&content) {
        Ok(s) => s,
        Err(e) => {
            return Ok(ImportResult {
                success: false,
                message: format!("Niepoprawny harmonogram: {e}"),
                year: None,
            });
        }
    };

    let target_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    let target_file = target_dir.join(format!("trash_schedule_{}.json", parsed.year));
    if let Err(e) = std::fs::write(&target_file, &content) {
        return Ok(ImportResult {
            success: false,
            message: format!("Błąd zapisu pliku: {e}"),
            year: None,
        });
    }

    Ok(ImportResult {
        success: true,
        message: format!(
            "Pomyślnie wczytano harmonogram na rok {} ({})",
            parsed.year, parsed.region
        ),
        year: Some(parsed.year),
    })
}

#[tauri::command]
fn ack_alarm(app: tauri::AppHandle) -> Result<(), String> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("config.json");
    let mut cfg = config::Config::load(&path)?;
    cfg.last_triggered_date = Some(chrono::Local::now().date_naive());
    cfg.save(&path)?;
    if let Some(w) = app.get_webview_window("alarm") {
        let _ = w.close();
    }
    Ok(())
}

#[tauri::command]
fn alarm_fractions(app: tauri::AppHandle) -> Result<Vec<FractionItem>, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    let cfg = config::Config::load(&dir.join("config.json"))?;
    let s = schedule::Schedules::load_dir(&dir)?;
    let now = chrono::Local::now().naive_local();
    let items = match scheduler::decide(now, &cfg, &s) {
        scheduler::Decision::Alarm(keys) => s
            .fractions_for(now.date())
            .into_iter()
            .filter(|(k, _)| keys.contains(&k.to_string()))
            .map(|(k, f)| FractionItem {
                key: k.to_string(),
                label: f.label.clone(),
            })
            .collect(),
        _ => {
            if let Some(next_d) = s.next_pickup(now.date()) {
                s.fractions_for(next_d)
                    .into_iter()
                    .map(|(k, f)| FractionItem {
                        key: k.to_string(),
                        label: f.label.clone(),
                    })
                    .collect()
            } else {
                vec![FractionItem {
                    key: "komunalne".to_string(),
                    label: "Zmieszane".to_string(),
                }]
            }
        }
    };
    Ok(items)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            // Druga kopia appki zamyka się bez alarmu
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let dir = app.path().app_config_dir()?;
            std::fs::create_dir_all(&dir)?;
            install_default_schedule_if_missing(&dir)?;

            let config_path = dir.join("config.json");
            let cfg = config::Config::load(&config_path).unwrap_or_default();

            let autolaunch = app.autolaunch();
            if cfg.autostart {
                if let Ok(false) = autolaunch.is_enabled() {
                    let _ = autolaunch.enable();
                }
            }

            let today = chrono::Local::now().date_naive();
            let schedules = schedule::Schedules::load_dir(&dir).unwrap_or_default();
            let next_pickup_text = match schedules.next_pickup(today) {
                Some(d) => {
                    let fr = schedules
                        .fractions_for(d)
                        .into_iter()
                        .map(|(_, f)| f.label.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    if fr.is_empty() {
                        format!("Następny wywóz: {d}")
                    } else {
                        format!("Następny wywóz: {d} ({fr})")
                    }
                }
                None => "Następny wywóz: brak".to_string(),
            };

            let next_item = MenuItemBuilder::with_id("next_pickup", next_pickup_text)
                .enabled(false)
                .build(app)?;
            let enabled_item = CheckMenuItemBuilder::with_id("toggle_enabled", "Włączony")
                .checked(cfg.enabled)
                .build(app)?;
            let autostart_item = CheckMenuItemBuilder::with_id("toggle_autostart", "Uruchamiaj z systemem")
                .checked(autolaunch.is_enabled().unwrap_or(cfg.autostart))
                .build(app)?;
            let test_item = MenuItemBuilder::with_id("test_alarm", "Testuj alarm").build(app)?;
            let options_item = MenuItemBuilder::with_id("options", "Opcje...").build(app)?;
            let import_item = MenuItemBuilder::with_id("import_json", "Wczytaj nowy JSON...").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Zakończ").build(app)?;

            let menu = MenuBuilder::new(app)
                .items(&[&next_item, &enabled_item, &autostart_item, &test_item, &options_item, &import_item, &quit])
                .build()?;

            TrayIconBuilder::with_id("main_tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Waste Reminder")
                .menu(&menu)
                .on_menu_event({
                    let config_path = config_path.clone();
                    move |app, event| {
                        match event.id().as_ref() {
                            "quit" => app.exit(0),
                            "toggle_enabled" => {
                                if let Ok(mut c) = config::Config::load(&config_path) {
                                    c.enabled = !c.enabled;
                                    let _ = c.save(&config_path);
                                }
                            }
                            "toggle_autostart" => {
                                let autolaunch = app.autolaunch();
                                let current = autolaunch.is_enabled().unwrap_or(false);
                                let next = !current;
                                if next {
                                    let _ = autolaunch.enable();
                                } else {
                                    let _ = autolaunch.disable();
                                }
                                if let Ok(mut c) = config::Config::load(&config_path) {
                                    c.autostart = next;
                                    let _ = c.save(&config_path);
                                }
                            }
                            "test_alarm" => {
                                let c = config::Config::load(&config_path).unwrap_or_default();
                                open_alarm_window(app, c.fullscreen);
                            }
                            "options" => {
                                let _ = open_options(app.clone());
                            }
                            "import_json" => {
                                open_expired_window(app);
                            }
                            _ => {}
                        }
                    }
                })
                .build(app)?;

            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut last_expired_shown_date: Option<chrono::NaiveDate> = None;
                loop {
                    let now = chrono::Local::now().naive_local();
                    let cfg = config::Config::load(&dir.join("config.json")).unwrap_or_default();
                    match schedule::Schedules::load_dir(&dir) {
                        Ok(s) => {
                            let d = scheduler::decide(now, &cfg, &s);
                            println!("[scheduler] {now} -> {:?}", d);
                            match d {
                                scheduler::Decision::Alarm(_) => {
                                    open_alarm_window(&handle, cfg.fullscreen);
                                }
                                scheduler::Decision::Expired => {
                                    if last_expired_shown_date != Some(now.date()) {
                                        last_expired_shown_date = Some(now.date());
                                        open_expired_window(&handle);
                                    }
                                }
                                scheduler::Decision::Nothing => {}
                            }

                            if let Some(tray) = handle.tray_by_id("main_tray") {
                                if scheduler::is_warning(now, &cfg, &s) {
                                    if let Some(last) = s.last_date() {
                                        let left = (last - now.date()).num_days();
                                        let _ = tray.set_tooltip(Some(format!(
                                            "Harmonogram kończy się za {left} dni"
                                        )));
                                    }
                                } else {
                                    let _ = tray.set_tooltip(Some("Waste Reminder".to_string()));
                                }
                            }
                        }
                        Err(e) => println!("[scheduler] błąd danych: {e}"),
                    }
                    std::thread::sleep(std::time::Duration::from_secs(30));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ack_alarm,
            alarm_fractions,
            open_options,
            get_config,
            save_config,
            get_available_sounds,
            pick_sound_file,
            close_window,
            import_schedule_dialog
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            // appka żyje w tray, zamknięcie okna alarmu jej nie kończy
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
