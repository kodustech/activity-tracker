use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use rusqlite::types::ToSql;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};
use std::path::PathBuf;

use crate::tracker::WindowActivity;

pub type DbConnection = Arc<Mutex<Connection>>;

fn get_database_path() -> Result<PathBuf> {
    let app_support = if cfg!(target_os = "macos") {
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

    std::fs::create_dir_all(&app_support)?;
    Ok(app_support.join("chronos.db"))
}

pub async fn init_database() -> Result<DbConnection> {
    info!("Initializing database");
    let db_path = get_database_path()?;
    info!("Database path: {:?}", db_path);
    
    let conn = Connection::open(db_path)?;
    
    // Habilita chaves estrangeiras e usa o modo DELETE para o journal
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = DELETE;"
    )?;
    
    info!("Creating table");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS activities (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            application TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            is_browser BOOLEAN NOT NULL,
            url TEXT,
            is_idle BOOLEAN NOT NULL DEFAULT 0
        )",
        [],
    )?;

    // Verifica se a coluna is_idle existe
    let columns: Vec<String> = conn
        .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='activities'")?
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;

    if let Some(create_sql) = columns.first() {
        if !create_sql.contains("is_idle") {
            info!("Adding is_idle column");
            conn.execute(
                "ALTER TABLE activities ADD COLUMN is_idle BOOLEAN NOT NULL DEFAULT 0",
                [],
            )?;
        }
    }

    info!("Database initialized successfully");
    Ok(Arc::new(Mutex::new(conn)))
}

pub async fn save_activity(conn: &DbConnection, activity: &WindowActivity) -> Result<i64> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare(
        "INSERT INTO activities (title, application, start_time, end_time, is_browser, url, is_idle)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    
    let id = stmt.insert([
        &activity.title as &dyn ToSql,
        &activity.application,
        &activity.start_time.to_rfc3339(),
        &activity.end_time.to_rfc3339(),
        &activity.is_browser,
        &activity.url,
        &activity.is_idle,
    ])?;
    
    Ok(id)
}

pub async fn get_activities_between(
    conn: &DbConnection,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<WindowActivity>> {
    let conn = conn.lock().await;
    debug!("Getting activities between {} and {}", start, end);
    
    let mut stmt = conn.prepare(
        r#"
        SELECT title, application, start_time, end_time, is_browser, url, is_idle
        FROM activities
        WHERE start_time >= ? AND end_time <= ?
        ORDER BY start_time DESC
        "#,
    )?;

    let activities = stmt
        .query_map(
            params![
                start.to_rfc3339(),
                end.to_rfc3339(),
            ],
            |row| {
                let start_time: String = row.get(2)?;
                let end_time: String = row.get(3)?;
                
                Ok(WindowActivity {
                    title: row.get(0)?,
                    application: row.get(1)?,
                    start_time: DateTime::parse_from_rfc3339(&start_time)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?.with_timezone(&Utc),
                    end_time: DateTime::parse_from_rfc3339(&end_time)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?.with_timezone(&Utc),
                    is_browser: row.get(4)?,
                    url: row.get(5)?,
                    is_idle: row.get(6).unwrap_or(false),
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;

    debug!("Found {} activities", activities.len());
    Ok(activities)
}

pub async fn merge_activity(
    conn: &DbConnection,
    activity: &WindowActivity,
    threshold_seconds: i64,
) -> Result<()> {
    let conn = conn.lock().await;
    
    info!(
        "🔍 Merging activity: {} - {} | Idle: {} | {} -> {}",
        activity.application,
        activity.title,
        activity.is_idle,
        activity.start_time.format("%H:%M:%S"),
        activity.end_time.format("%H:%M:%S")
    );

    // Primeiro tenta encontrar uma atividade similar recente
    let similar: Option<(i64, DateTime<Utc>, bool)> = conn
        .query_row(
            r#"
            SELECT id, end_time, is_idle
            FROM activities
            WHERE application = ?
              AND title = ?
              AND is_browser = ?
              AND is_idle = ?  -- Só mescla se o estado de idle for o mesmo
              AND date(start_time) = date(?)
              AND (strftime('%s', ?) - strftime('%s', end_time)) <= ?
            ORDER BY end_time DESC
            LIMIT 1
            "#,
            params![
                activity.application,
                activity.title,
                activity.is_browser,
                activity.is_idle,
                activity.start_time.to_rfc3339(),
                activity.start_time.to_rfc3339(),
                threshold_seconds,
            ],
            |row| {
                let end_time: String = row.get(1)?;
                Ok((
                    row.get(0)?,
                    DateTime::parse_from_rfc3339(&end_time)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?.with_timezone(&Utc),
                    row.get(2)?,
                ))
            },
        )
        .optional()?;

    if let Some((id, end_time, is_idle)) = similar {
        info!(
            "🔄 Updating activity {} | Idle: {} -> {} | End: {} -> {}",
            id,
            is_idle,
            activity.is_idle,
            end_time.format("%H:%M:%S"),
            activity.end_time.format("%H:%M:%S")
        );
        
        conn.execute(
            "UPDATE activities SET end_time = ? WHERE id = ?",
            params![activity.end_time.to_rfc3339(), id],
        )?;
    } else {
        info!(
            "➕ New activity | Idle: {} | {} -> {}",
            activity.is_idle,
            activity.start_time.format("%H:%M:%S"),
            activity.end_time.format("%H:%M:%S")
        );
        
        conn.execute(
            r#"
            INSERT INTO activities (
                title, application, start_time, end_time, 
                is_browser, url, is_idle
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                activity.title,
                activity.application,
                activity.start_time.to_rfc3339(),
                activity.end_time.to_rfc3339(),
                activity.is_browser,
                activity.url,
                activity.is_idle,
            ],
        )?;
    }

    Ok(())
}

pub async fn get_activities_for_day(
    conn: &DbConnection,
    date: DateTime<Utc>,
) -> Result<Vec<WindowActivity>> {
    let conn = conn.lock().await;
    debug!("Getting activities for day {}", date.date_naive());
    
    let mut stmt = conn.prepare(
        r#"
        SELECT title, application, start_time, end_time, is_browser, url, is_idle
        FROM activities
        WHERE date(start_time) = date(?)
        ORDER BY start_time DESC
        "#,
    )?;

    let activities = stmt
        .query_map(
            params![date.to_rfc3339()],
            |row| {
                let start_time: String = row.get(2)?;
                let end_time: String = row.get(3)?;
                
                Ok(WindowActivity {
                    title: row.get(0)?,
                    application: row.get(1)?,
                    start_time: DateTime::parse_from_rfc3339(&start_time)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?.with_timezone(&Utc),
                    end_time: DateTime::parse_from_rfc3339(&end_time)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?.with_timezone(&Utc),
                    is_browser: row.get(4)?,
                    url: row.get(5)?,
                    is_idle: row.get(6).unwrap_or(false),
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;

    debug!("Found {} activities for day {}", activities.len(), date.date_naive());
    Ok(activities)
}

pub async fn get_unique_applications(conn: &DbConnection) -> Result<Vec<String>> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare("SELECT DISTINCT application FROM activities")?;
    let apps = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(apps)
} 