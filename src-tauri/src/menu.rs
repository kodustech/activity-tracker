use image::{ImageBuffer, Rgba, RgbaImage};
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};
use std::sync::Mutex;
use tracing::info;
use crate::database::DbConnection;
use crate::category::CategoryConfig;
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};

fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

fn generate_dynamic_icon(value: i64) -> Result<Vec<u8>, String> {
    // Configurações do ícone
    let width = 22;
    let height = 22;
    
    // Cria imagem transparente
    let mut img: RgbaImage = ImageBuffer::new(width, height);
    
    // Preenche com pixels transparentes
    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }
    
    // Desenha um número simples
    let text = format!("{}%", value);
    let x = 2;
    let y = 2;
    
    // Desenha cada caractere como pixels
    for (i, c) in text.chars().enumerate() {
        let offset = i as u32 * 6;
        match c {
            '0'..='9' => {
                for dx in 0..5 {
                    for dy in 0..7 {
                        img.put_pixel(x + offset + dx, y + dy, Rgba([255, 255, 255, 255]));
                    }
                }
            },
            '%' => {
                for dx in 0..5 {
                    img.put_pixel(x + offset + dx, y + dx, Rgba([255, 255, 255, 255]));
                }
            },
            _ => {}
        }
    }
    
    // Converte para RGBA raw bytes
    let raw_data: Vec<u8> = img.pixels().flat_map(|p| p.0.to_vec()).collect();
    
    Ok(raw_data)
}

pub fn create_tray_menu() -> SystemTray {
    let tracked = CustomMenuItem::new("tracked".to_string(), "Tracked: --");
    let productive = CustomMenuItem::new("productive".to_string(), "Productive: --");
    let progress = CustomMenuItem::new("progress".to_string(), "▱▱▱▱▱▱▱▱▱▱ 0%");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    
    let tray_menu = SystemTrayMenu::new()
        .add_item(progress.disabled())
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(tracked.disabled())
        .add_item(productive.disabled())
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    SystemTray::new()
        .with_menu(tray_menu)
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

fn create_progress_bar(percentage: i64) -> String {
    let filled = (percentage as f64 / 100.0 * 10.0).round() as usize;
    let empty = 10 - filled;
    
    let filled_chars = "▰".repeat(filled);
    let empty_chars = "▱".repeat(empty);
    
    format!("{}{} {}%", filled_chars, empty_chars, percentage)
}

pub async fn update_tray_menu(app: &AppHandle) -> Result<(), String> {
    info!("Updating tray menu");
    
    // Get today's stats using the internal function directly
    let db = app.state::<DbConnection>();
    let config = app.state::<Mutex<CategoryConfig>>();
    let config_clone = config.clone();
    
    let (total_minutes, productive_minutes) = match crate::commands::get_today_stats_internal(db, config).await {
        Ok((total, productive)) => {
            let total_minutes = total / 60;
            let productive_minutes = productive / 60;
            (total_minutes, productive_minutes)
        },
        Err(e) => {
            info!("Error getting today's stats: {}", e);
            (0, 0)
        }
    };
    
    // Calculate goal percentage
    let goal_percentage = if let Ok(config) = config_clone.inner().lock() {
        if config.daily_goal_minutes > 0 {
            ((productive_minutes as f64 / config.daily_goal_minutes as f64) * 100.0).round() as i64
        } else {
            0
        }
    } else {
        info!("Failed to lock config");
        0
    };
    
    // Format durations
    let tracked = CustomMenuItem::new("tracked", format!("Tracked: {}", format_duration(total_minutes * 60)));
    let productive = CustomMenuItem::new("productive", format!("Productive: {} ({}%)", format_duration(productive_minutes * 60), goal_percentage));
    let progress = CustomMenuItem::new("progress", format!("🎯 {}%", goal_percentage));
    let quit = CustomMenuItem::new("quit", "Quit");
    
    // Create menu
    let tray_menu = SystemTrayMenu::new()
        .add_item(progress.disabled())
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(tracked.disabled())
        .add_item(productive.disabled())
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    
    // Update the menu
    let tray_handle = app.tray_handle();
    tray_handle.set_menu(tray_menu).map_err(|e| e.to_string())?;
    
    // Update the title with percentage
    let title = format!("{}%", goal_percentage);
    info!("Setting tray title to: {}", title);
    if let Err(e) = tray_handle.set_title(&title) {
        info!("Failed to set tray title: {}", e);
    }
    
    Ok(())
} 