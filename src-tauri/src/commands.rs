use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;
use std::collections::HashSet;
use tracing::{info, error};

use crate::database::{self, DbConnection};
use crate::tracker::WindowActivity;
use crate::category::{Category, CategoryConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeRange {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DailyStats {
    total_time: i64,
    productive_time: i64,
    top_applications: Vec<ApplicationStats>,
    activities: Vec<WindowActivity>,
}

#[derive(Debug, Serialize)]
pub struct ApplicationStats {
    application: String,
    total_duration: i64,
    activities: Vec<WindowActivity>,
    category: Option<Category>,
}

#[tauri::command]
pub async fn get_activities(
    range: TimeRange,
    db: State<'_, DbConnection>,
) -> Result<Vec<WindowActivity>, String> {
    database::get_activities_between(&db, range.start, range.end)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_daily_stats(
    date: String,
    db: State<'_, DbConnection>,
    config: State<'_, Mutex<CategoryConfig>>,
) -> Result<DailyStats, String> {
    let date = DateTime::parse_from_rfc3339(&date)
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);
    
    let start = date.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let end = date.date_naive().and_hms_opt(23, 59, 59).unwrap();
    
    let activities = database::get_activities_between(&db, start.and_utc(), end.and_utc())
        .await
        .map_err(|e| e.to_string())?;

    let config = config.lock().map_err(|e| e.to_string())?;

    // Agrupa atividades por aplicativo
    let mut app_stats: std::collections::HashMap<String, Vec<WindowActivity>> = std::collections::HashMap::new();
    for activity in activities.iter() {
        app_stats.entry(activity.application.clone())
            .or_default()
            .push(activity.clone());
    }

    // Calcula estatísticas por aplicativo
    let mut top_applications: Vec<ApplicationStats> = app_stats
        .into_iter()
        .map(|(app, activities)| {
            let total_duration = activities.iter()
                .map(|a| (a.end_time - a.start_time).num_seconds())
                .sum();
            
            let category = config.get_category_for_app(&app).cloned();
            info!(
                "App: {}, Category: {:?}, Duration: {}",
                app,
                category.as_ref().map(|c| &c.name),
                total_duration
            );
            
            ApplicationStats {
                application: app,
                total_duration,
                activities,
                category,
            }
        })
        .collect();

    // Ordena por duração total
    top_applications.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));

    // Calcula tempos totais
    let total_time: i64 = top_applications.iter()
        .map(|app| app.total_duration)
        .sum();

    let productive_time: i64 = top_applications.iter()
        .filter(|app| {
            let is_productive = app.category.as_ref().map_or(false, |c| c.is_productive);
            info!(
                "App: {}, Duration: {}, Category: {:?}, Is Productive: {}",
                app.application,
                app.total_duration,
                app.category.as_ref().map(|c| &c.name),
                is_productive
            );
            is_productive
        })
        .map(|app| app.total_duration)
        .sum();

    info!("Total time: {}, Productive time: {}", total_time, productive_time);

    Ok(DailyStats {
        total_time,
        productive_time,
        top_applications: top_applications.into_iter().take(5).collect(),
        activities,
    })
}

fn is_unproductive_app(app_name: &str) -> bool {
    const UNPRODUCTIVE_APPS: &[&str] = &[
        "Finder",
        "System Settings",
        "System Preferences",
        "Notification Center",
        "Dock",
        "Spotlight",
        "Menu Bar",
    ];

    UNPRODUCTIVE_APPS.contains(&app_name)
}

#[tauri::command]
pub async fn get_activities_for_day(
    state: tauri::State<'_, DbConnection>,
    date: String,
) -> Result<Vec<WindowActivity>, String> {
    let date = DateTime::parse_from_rfc3339(&date)
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);
    
    database::get_activities_for_day(&state, date)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories(
    config: State<'_, Mutex<CategoryConfig>>,
) -> Result<Vec<Category>, String> {
    let config = config.lock().map_err(|e| e.to_string())?;
    Ok(config.categories.clone())
}

#[tauri::command]
pub async fn get_app_categories(
    config: State<'_, Mutex<CategoryConfig>>,
) -> Result<Vec<(String, String)>, String> {
    let config = config.lock().map_err(|e| e.to_string())?;
    Ok(config.app_categories
        .iter()
        .map(|(app, cat)| (app.clone(), cat.clone()))
        .collect())
}

#[tauri::command]
pub async fn add_category(
    config: State<'_, Mutex<CategoryConfig>>,
    name: String,
    color: String,
    is_productive: bool,
) -> Result<Category, String> {
    let mut config = config.lock().map_err(|e| e.to_string())?;
    config.add_category(name, color, is_productive)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_category(
    config: State<'_, Mutex<CategoryConfig>>,
    id: String,
    name: String,
    color: String,
    is_productive: bool,
) -> Result<(), String> {
    let mut config = config.lock().map_err(|e| e.to_string())?;
    config.update_category(id, name, color, is_productive)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_category(
    config: State<'_, Mutex<CategoryConfig>>,
    id: String,
) -> Result<(), String> {
    let mut config = config.lock().map_err(|e| e.to_string())?;
    config.delete_category(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_app_category(
    state: State<'_, Mutex<CategoryConfig>>,
    app_name: String,
    category_id: String,
) -> Result<(), String> {
    info!("Received request to set category. App: '{}', Category ID: '{}'", app_name, category_id);
    
    let mut config = match state.lock() {
        Ok(config) => {
            info!("Successfully acquired lock on config");
            config
        },
        Err(e) => {
            error!("Failed to acquire lock on config: {}", e);
            return Err(e.to_string());
        }
    };
    
    info!("Current categories: {:?}", config.categories);
    info!("Current app categories: {:?}", config.app_categories);
    
    match config.set_app_category(app_name, category_id) {
        Ok(()) => {
            info!("Successfully set app category");
            Ok(())
        },
        Err(e) => {
            error!("Failed to set app category: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_uncategorized_apps(
    db: State<'_, DbConnection>,
    config: State<'_, Mutex<CategoryConfig>>,
) -> Result<Vec<String>, String> {
    // Busca todos os aplicativos únicos do banco
    let apps = database::get_unique_applications(&db)
        .await
        .map_err(|e| e.to_string())?;

    // Pega os aplicativos que já têm categoria
    let config = config.lock().map_err(|e| e.to_string())?;
    let categorized_apps: HashSet<_> = config.app_categories.keys().cloned().collect();

    // Filtra apenas os apps não categorizados
    let uncategorized = apps
        .into_iter()
        .filter(|app| !categorized_apps.contains(app))
        .collect();

    Ok(uncategorized)
}

#[tauri::command]
pub async fn get_today_stats(
    db: State<'_, DbConnection>,
    config: State<'_, Mutex<CategoryConfig>>,
) -> Result<(i64, i64), String> {
    let now = Utc::now();
    let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();
    
    let activities = database::get_activities_between(&db, start.and_utc(), end.and_utc())
        .await
        .map_err(|e| e.to_string())?;

    let config = config.lock().map_err(|e| e.to_string())?;

    // Agrupa atividades por aplicativo
    let mut app_stats: std::collections::HashMap<String, Vec<WindowActivity>> = std::collections::HashMap::new();
    for activity in activities.iter() {
        app_stats.entry(activity.application.clone())
            .or_default()
            .push(activity.clone());
    }

    // Calcula estatísticas por aplicativo
    let top_applications: Vec<ApplicationStats> = app_stats
        .into_iter()
        .map(|(app, activities)| {
            let total_duration = activities.iter()
                .map(|a| (a.end_time - a.start_time).num_seconds())
                .sum();
            
            let category = config.get_category_for_app(&app).cloned();
            
            ApplicationStats {
                application: app,
                total_duration,
                activities,
                category,
            }
        })
        .collect();

    // Calcula tempos totais
    let total_time: i64 = top_applications.iter()
        .map(|app| app.total_duration)
        .sum();

    let productive_time: i64 = top_applications.iter()
        .filter(|app| app.category.as_ref().map_or(false, |c| c.is_productive))
        .map(|app| app.total_duration)
        .sum();

    Ok((total_time, productive_time))
} 