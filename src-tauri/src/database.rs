use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::tracker::WindowActivity;

pub type DbConnection = Arc<Mutex<Connection>>;

pub async fn init_database() -> Result<DbConnection> {
    info!("Initializing database");
    let conn = Connection::open("chronos.db")?;
    
    // Habilita chaves estrangeiras e usa o modo DELETE para o journal
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = DELETE;"
    )?;
    
    info!("Creating table");
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS activities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            application TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            is_browser BOOLEAN NOT NULL,
            url TEXT
        );
        "#,
    )?;

    info!("Database initialized successfully");
    Ok(Arc::new(Mutex::new(conn)))
}

pub async fn save_activity(conn: &DbConnection, activity: &WindowActivity) -> Result<i64> {
    let conn = conn.lock().await;
    let day = activity.start_time.date_naive().to_string();
    
    debug!("Saving activity: {} - {}", activity.application, activity.title);
    conn.execute(
        r#"
        INSERT INTO activities (
            title, application, start_time, end_time, 
            is_browser, url, day
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
            day,
        ],
    )?;

    Ok(conn.last_insert_rowid())
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
        SELECT title, application, start_time, end_time, is_browser, url
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
    
    debug!(
        "Trying to merge activity: {} - {}",
        activity.application, activity.title
    );

    let similar: Option<(i64, DateTime<Utc>)> = conn
        .query_row(
            r#"
            SELECT id, end_time
            FROM activities
            WHERE application = ?
              AND title = ?
              AND is_browser = ?
              AND date(start_time) = date(?)
              AND (strftime('%s', ?) - strftime('%s', end_time)) <= ?
            ORDER BY end_time DESC
            LIMIT 1
            "#,
            params![
                activity.application,
                activity.title,
                activity.is_browser,
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
                ))
            },
        )
        .optional()?;

    if let Some((id, _)) = similar {
        debug!("Updating existing activity {}", id);
        conn.execute(
            "UPDATE activities SET end_time = ? WHERE id = ?",
            params![activity.end_time.to_rfc3339(), id],
        )?;
    } else {
        debug!("Creating new activity");
        conn.execute(
            r#"
            INSERT INTO activities (
                title, application, start_time, end_time, 
                is_browser, url
            )
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            params![
                activity.title,
                activity.application,
                activity.start_time.to_rfc3339(),
                activity.end_time.to_rfc3339(),
                activity.is_browser,
                activity.url,
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
        SELECT title, application, start_time, end_time, is_browser, url
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