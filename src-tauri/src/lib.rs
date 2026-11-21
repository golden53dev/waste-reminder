use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::ManagerExt;
mod config;
mod schedule;
mod scheduler;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct FractionItem {
    pub key: String,
    pub label: String,
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
        .inner_size(460.0, 520.0)
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
            let quit = MenuItemBuilder::with_id("quit", "Zakończ").build(app)?;

            let menu = MenuBuilder::new(app)
                .items(&[&next_item, &enabled_item, &autostart_item, &test_item, &options_item, &quit])
                .build()?;

            TrayIconBuilder::new()
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
                            _ => {}
                        }
                    }
                })
                .build(app)?;

            let handle = app.handle().clone();
            std::thread::spawn(move || {
                loop {
                    let now = chrono::Local::now().naive_local();
                    let cfg = config::Config::load(&dir.join("config.json")).unwrap_or_default();
                    match schedule::Schedules::load_dir(&dir) {
                        Ok(s) => {
                            let d = scheduler::decide(now, &cfg, &s);
                            println!("[scheduler] {now} -> {:?}", d);
                            if let scheduler::Decision::Alarm(_) = d {
                                open_alarm_window(&handle, cfg.fullscreen);
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
            get_config
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
