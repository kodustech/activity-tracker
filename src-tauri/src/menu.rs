use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray,
    SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
};
use chrono::{Utc, Duration};
use serde_json::json;

pub fn create_tray_menu() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show = CustomMenuItem::new("show".to_string(), "Show Window");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide Window");
    let today = CustomMenuItem::new("today".to_string(), "Today's Stats");
    let yesterday = CustomMenuItem::new("yesterday".to_string(), "Yesterday's Stats");
    let last_week = CustomMenuItem::new("last_week".to_string(), "Last 7 Days");
    let last_month = CustomMenuItem::new("last_month".to_string(), "Last 30 Days");
    let pause = CustomMenuItem::new("pause".to_string(), "Pause Tracking");
    
    let tray_menu = SystemTrayMenu::new()
        .add_item(today)
        .add_item(yesterday)
        .add_item(last_week)
        .add_item(last_month)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(show)
        .add_item(hide)
        .add_item(pause)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    SystemTray::new().with_menu(tray_menu)
}

pub fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => {
            let window = app.get_window("main").unwrap();
            
            match id.as_str() {
                "quit" => {
                    app.exit(0);
                }
                "show" => {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "hide" => {
                    window.hide().unwrap();
                }
                "today" => {
                    let _ = window.emit("show_stats", Utc::now().date_naive().to_string());
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "yesterday" => {
                    let yesterday = (Utc::now() - Duration::days(1)).date_naive().to_string();
                    let _ = window.emit("show_stats", yesterday);
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "last_week" => {
                    let _ = window.emit("show_range", json!({
                        "start": (Utc::now() - Duration::days(7)).date_naive().to_string(),
                        "end": Utc::now().date_naive().to_string()
                    }));
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "last_month" => {
                    let _ = window.emit("show_range", json!({
                        "start": (Utc::now() - Duration::days(30)).date_naive().to_string(),
                        "end": Utc::now().date_naive().to_string()
                    }));
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "pause" => {
                    // TODO: Implementar pausa no rastreamento
                }
                _ => {}
            }
        }
        SystemTrayEvent::LeftClick { .. } => {
            let window = app.get_window("main").unwrap();
            if window.is_visible().unwrap() {
                window.hide().unwrap();
            } else {
                window.show().unwrap();
                window.set_focus().unwrap();
            }
        }
        _ => {}
    }
} 