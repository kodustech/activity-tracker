// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod tracker;
mod commands;
mod menu;
mod category;

use anyhow::Result;
use tauri::Manager;
use tracing::{info, error};
use std::sync::Mutex;
use category::CategoryConfig;

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

    // Carrega a configuração de categorias
    let category_config = match CategoryConfig::load() {
        Ok(config) => {
            info!("Category configuration loaded successfully with {} categories", config.categories.len());
            info!("Categories: {:?}", config.categories);
            config
        },
        Err(e) => {
            error!("Failed to load category configuration: {}", e);
            info!("Creating default configuration");
            CategoryConfig::default()
        }
    };

    // Inicia a aplicação Tauri
    tauri::Builder::default()
        .manage(db_for_state)
        .manage(Mutex::new(category_config))
        .system_tray(menu::create_tray_menu())
        .on_system_tray_event(menu::handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            commands::get_activities,
            commands::get_daily_stats,
            commands::get_activities_for_day,
            commands::get_categories,
            commands::get_app_categories,
            commands::add_category,
            commands::update_category,
            commands::delete_category,
            commands::set_app_category,
            commands::get_uncategorized_apps,
            commands::get_today_stats,
            commands::get_daily_goal,
            commands::set_daily_goal,
        ])
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            window.set_title("Chronos Track").unwrap();

            // Atualiza o menu da top bar periodicamente
            let app_handle = app.handle();
            tokio::spawn(async move {
                // Atualiza imediatamente ao iniciar
                menu::update_tray_menu(&app_handle).await;
                
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    menu::update_tray_menu(&app_handle).await;
                }
            });

            Ok(())
        })
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                event.window().hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
