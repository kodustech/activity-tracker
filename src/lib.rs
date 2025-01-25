mod tracker;
mod commands;
mod category;
mod database;
mod menu;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
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