use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray,
    SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
};
use chrono::Utc;
use std::sync::Mutex;
use std::time::Duration as StdDuration;
use crate::database::DbConnection;
use crate::category::CategoryConfig;
use crate::commands;

fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

pub fn create_tray_menu() -> SystemTray {
    let show = CustomMenuItem::new("show".to_string(), "Show");
    let tracked = CustomMenuItem::new("tracked".to_string(), "Tracked: --");
    let productive = CustomMenuItem::new("productive".to_string(), "Productive: --");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    
    let tray_menu = SystemTrayMenu::new()
        .add_item(tracked.disabled())
        .add_item(productive.disabled())
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(show)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    SystemTray::new().with_menu(tray_menu)
}

pub fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick {
            position: _,
            size: _,
            ..
        } => {
            let window = app.get_window("main").unwrap();
            if window.is_visible().unwrap() {
                window.hide().unwrap();
            } else {
                window.show().unwrap();
                window.set_focus().unwrap();
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "show" => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        },
        _ => {}
    }
}

pub async fn update_tray_menu(app: &AppHandle) {
    let db = app.state::<DbConnection>();
    let config = app.state::<Mutex<CategoryConfig>>();
    
    if let Ok((total_time, productive_time)) = commands::get_today_stats(db, config).await {
        let tracked = CustomMenuItem::new(
            "tracked".to_string(),
            format!("Tracked: {}", format_duration(total_time))
        ).disabled();
        
        let productive = CustomMenuItem::new(
            "productive".to_string(),
            format!("Productive: {}", format_duration(productive_time))
        ).disabled();
        
        let show = CustomMenuItem::new("show".to_string(), "Show");
        let quit = CustomMenuItem::new("quit".to_string(), "Quit");
        
        let tray_menu = SystemTrayMenu::new()
            .add_item(tracked)
            .add_item(productive)
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(show)
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(quit);

        app.tray_handle().set_menu(tray_menu).unwrap();
    }
} 