// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use anyhow::Result;
use tracing::info;

mod database;
mod tracker;
mod commands;
mod category;
pub mod menu;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Inicializa o logger
    tracing_subscriber::fmt::init();
    info!("Starting Chronos Track");

    tauri::Builder::default()
        .setup(|_app| {
            // Inicializa o banco de dados e o rastreador em uma nova thread
            tauri::async_runtime::spawn(async move {
                match init_tracking().await {
                    Ok(_) => info!("Tracking system initialized successfully"),
                    Err(e) => eprintln!("Failed to initialize tracking system: {}", e),
                }
            });

            Ok(())
        })
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn init_tracking() -> Result<()> {
    // Inicializa o banco de dados
    let db = database::init_database().await?;
    
    // Inicializa o rastreador
    let mut tracker = tracker::ActivityTracker::new(db).await;
    
    // Inicia o rastreamento
    tracker.start_tracking().await;
    
    Ok(())
}
