// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod tracker;
mod commands;
mod menu;
mod category;

use anyhow::Result;
use tauri::Manager;
use tracing::{info, error, debug, warn};
use std::sync::Mutex;
use category::CategoryConfig;
use std::path::PathBuf;

fn get_app_dir() -> Result<PathBuf> {
    let app_dir = if cfg!(target_os = "macos") {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join("Library")
            .join("Application Support")
            .join("com.chronos.track")
    } else if cfg!(target_os = "windows") {
        dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find local data directory"))?
            .join("com.chronos.track")
    } else {
        dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?
            .join("com.chronos.track")
    };

    std::fs::create_dir_all(&app_dir)?;
    Ok(app_dir)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Configura o logger para escrever em um arquivo
    let app_dir = get_app_dir()?;
    let log_dir = app_dir.join("logs");
    std::fs::create_dir_all(&log_dir)?;
    
    let file_appender = tracing_appender::rolling::RollingFileAppender::new(
        tracing_appender::rolling::Rotation::NEVER,
        log_dir,
        "chronos-track.log",
    );
    
    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_line_number(true)
        .with_file(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(false)
        .init();

    info!("Starting Chronos Track");
    debug!("Initializing application...");
    debug!("App directory: {:?}", app_dir);

    // Inicializa o banco de dados
    debug!("Initializing database...");
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
    debug!("Initializing activity tracker...");
    let mut tracker = tracker::ActivityTracker::new(db).await;
    info!("Activity tracker initialized successfully");
    
    // Inicia o rastreamento em uma nova thread
    tokio::spawn(async move {
        info!("Starting activity tracking");
        tracker.start_tracking().await;
        error!("Activity tracking loop ended unexpectedly");
    });

    // Carrega a configuração de categorias
    debug!("Loading category configuration...");
    let category_config = match CategoryConfig::load() {
        Ok(config) => {
            info!("Category configuration loaded successfully with {} categories", config.categories.len());
            debug!("Categories: {:?}", config.categories);
            config
        },
        Err(e) => {
            warn!("Failed to load category configuration: {}", e);
            info!("Creating default configuration");
            CategoryConfig::default()
        }
    };

    // Inicia a aplicação Tauri
    debug!("Starting Tauri application...");
    let app = tauri::Builder::default()
        .manage(db_for_state)
        .manage(Mutex::new(category_config))
        .system_tray(menu::create_tray_menu())
        .on_system_tray_event(menu::handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            commands::get_activities,
            commands::get_daily_stats,
            commands::get_weekly_stats,
            commands::get_monthly_stats,
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
            debug!("Setting up main window...");
            let window = match app.get_window("main") {
                Some(window) => window,
                None => {
                    error!("Failed to get main window");
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Failed to get main window"
                    )));
                }
            };

            if let Err(e) = window.set_title("Chronos Track") {
                error!("Failed to set window title: {}", e);
            }

            debug!("Setting up tray menu updater...");
            let app_handle = app.handle();
            tokio::spawn(async move {
                debug!("Starting tray menu update loop");
                if let Err(e) = menu::update_tray_menu(&app_handle).await {
                    error!("Failed to update tray menu: {}", e);
                }
                
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
                loop {
                    interval.tick().await;
                    if let Err(e) = menu::update_tray_menu(&app_handle).await {
                        error!("Failed to update tray menu: {}", e);
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|event| {
            debug!("Window event received: {:?}", event.event());
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                if let Err(e) = event.window().hide() {
                    error!("Failed to hide window: {}", e);
                }
                api.prevent_close();
            }
        });

    debug!("Running Tauri application...");
    match app.run(tauri::generate_context!()) {
        Ok(_) => {
            info!("Application exited successfully");
            Ok(())
        },
        Err(e) => {
            error!("Application failed to run: {}", e);
            Err(e.into())
        }
    }
}
