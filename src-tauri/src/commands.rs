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
    pub total_time: i64,
    pub productive_time: i64,
    pub goal_percentage: i64,
    pub top_applications: Vec<ApplicationStats>,
    pub activities: Vec<WindowActivity>,
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

    // Calcula a porcentagem da meta
    let productive_minutes = productive_time / 60;
    let goal_percentage = if config.daily_goal_minutes > 0 {
        ((productive_minutes as f64 / config.daily_goal_minutes as f64) * 100.0).round() as i64
    } else {
        0
    };

    info!("Total time: {}, Productive time: {}, Goal: {}%", total_time, productive_time, goal_percentage);

    Ok(DailyStats {
        total_time,
        productive_time,
        goal_percentage,
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
    app: tauri::AppHandle,
    state: State<'_, Mutex<CategoryConfig>>,
    app_name: String,
    category_id: String,
) -> Result<(), String> {
    info!("Received request to set category. App: '{}', Category ID: '{}'", app_name, category_id);
    
    // Faz a alteração dentro de um escopo para garantir que o lock é liberado
    {
        let mut config = state.lock().map_err(|e| e.to_string())?;
        config.set_app_category(app_name, category_id).map_err(|e| e.to_string())?;
    } // lock é liberado aqui
    
    // Spawn a new task to update the menu
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = Box::pin(crate::menu::update_tray_menu(&app_handle)).await {
            error!("Failed to update menu: {}", e);
        }
    });
    
    Ok(())
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
    app: tauri::AppHandle,
    db: State<'_, DbConnection>,
    config: State<'_, Mutex<CategoryConfig>>,
) -> Result<(i64, i64), String> {
    let result = get_today_stats_internal(db, config).await?;
    
    // Atualiza o menu em uma nova task
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::menu::update_tray_menu(&app_handle).await {
            error!("Failed to update menu: {}", e);
        }
    });
    
    Ok(result)
}

pub async fn get_today_stats_internal(
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

async fn get_category_config() -> Result<CategoryConfig, String> {
    CategoryConfig::load().map_err(|e| e.to_string())
}

async fn save_category_config(config: &CategoryConfig) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_daily_goal() -> Result<i64, String> {
    let config = get_category_config().await?;
    Ok(config.daily_goal_minutes)
}

#[tauri::command]
pub async fn set_daily_goal(
    app: tauri::AppHandle,
    minutes: i64
) -> Result<(), String> {
    let mut config = get_category_config().await?;
    config.daily_goal_minutes = minutes;
    save_category_config(&config).await?;
    
    // Atualiza o menu
    crate::menu::update_tray_menu(&app).await;
    
    Ok(())
} 