use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
mod config;
mod schedule;
mod scheduler;

//#[tauri::command]
//fn greet(name: &str) -> String {
//  format!("Hello, {}! You've been greeted from Rust!", name)
//}

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
fn alarm_fractions(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    let cfg = config::Config::load(&dir.join("config.json"))?;
    let s = schedule::Schedules::load_dir(&dir)?;
    let now = chrono::Local::now().naive_local();
    Ok(match scheduler::decide(now, &cfg, &s) {
        scheduler::Decision::Alarm(k) => k,
        _ => vec![],
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            std::fs::create_dir_all(&dir)?;

            let status = MenuItemBuilder::with_id("status", "Waste Reminder: aktywny")
                .enabled(false)
                .build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Zakończ").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&status, &quit]).build()?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Waste Reminder")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if event.id() == "quit" {
                        app.exit(0);
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
                                if handle.get_webview_window("alarm").is_none() {
                                    let mut b = WebviewWindowBuilder::new(
                                        &handle,
                                        "alarm",
                                        WebviewUrl::App("alarm.html".into()),
                                    )
                                    .always_on_top(true)
                                    .decorations(false)
                                    .transparent(true);
                                    b = if cfg.fullscreen {
                                        b.fullscreen(true)
                                    } else {
                                        b.inner_size(520.0, 420.0).resizable(false).center()
                                    };
                                    let _ = b.build();
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
        .invoke_handler(tauri::generate_handler![ack_alarm, alarm_fractions])
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
