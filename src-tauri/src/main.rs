// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod tracker;
mod commands;
mod menu;

use anyhow::Result;
use tauri::Manager;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializa o logger
    tracing_subscriber::fmt::init();
    info!("Starting Chronos Track");

    // Inicializa o banco de dados
    let db = match database::init_database().await {
        Ok(db) => {
            info!("Database initialized successfully");
            db
        },
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            return Err(e);
        }
    };

    let db_for_state = db.clone();
    
    // Inicializa o rastreador
    let mut tracker = tracker::ActivityTracker::new(db).await;
    
    // Inicia o rastreamento em uma nova thread
    tokio::spawn(async move {
        info!("Starting activity tracking");
        tracker.start_tracking().await;
    });

    // Inicia a aplicação Tauri
    tauri::Builder::default()
        .manage(db_for_state)
        .system_tray(menu::create_tray_menu())
        .on_system_tray_event(menu::handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            commands::get_activities,
            commands::get_daily_stats,
            commands::get_activities_for_day,
        ])
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            window.set_title("Chronos Track").unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
