use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::database::{self, DbConnection};
use crate::tracker::WindowActivity;

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
) -> Result<DailyStats, String> {
    let date = DateTime::parse_from_rfc3339(&date)
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);
    
    let start = date.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let end = date.date_naive().and_hms_opt(23, 59, 59).unwrap();
    
    let activities = database::get_activities_between(&db, start.and_utc(), end.and_utc())
        .await
        .map_err(|e| e.to_string())?;

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
            
            ApplicationStats {
                application: app,
                total_duration,
                activities,
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
        .filter(|app| !is_unproductive_app(&app.application))
        .map(|app| app.total_duration)
        .sum();

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