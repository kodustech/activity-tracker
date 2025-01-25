use active_win_pos_rs::get_active_window;
use anyhow::Error as AnyhowError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time;
use tracing::{debug, error, info};

use crate::database::{self, DbConnection};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowActivity {
    pub title: String,
    pub application: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_browser: bool,
    pub url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum TrackerError {
    #[error("Failed to get active window")]
    WindowError(()),
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Other error: {0}")]
    Other(#[from] AnyhowError),
}

pub struct ActivityTracker {
    db: DbConnection,
    current_window: Option<WindowActivity>,
}

impl ActivityTracker {
    pub async fn new(db: DbConnection) -> Self {
        Self {
            db,
            current_window: None,
        }
    }

    pub async fn start_tracking(&mut self) {
        info!("Starting activity tracking");
        let mut interval = time::interval(Duration::from_secs(15));

        loop {
            interval.tick().await;
            match self.track_current_window().await {
                Ok(_) => debug!("Successfully tracked window"),
                Err(e) => error!("Error tracking window: {}", e),
            }
        }
    }

    async fn track_current_window(&mut self) -> Result<(), TrackerError> {
        let window = get_active_window().map_err(|_| TrackerError::WindowError(()))?;
        
        let now = Utc::now();
        let activity = WindowActivity {
            title: window.title,
            application: window.app_name,
            start_time: now,
            end_time: now,
            is_browser: false, // TODO: Implement browser detection
            url: None,
        };

        // Tenta mesclar com atividade similar ou salva uma nova
        database::merge_activity(&self.db, &activity, 300)
            .await
            .map_err(AnyhowError::from)?;
        
        debug!("Tracked window: {:?}", activity);
        self.current_window = Some(activity);
        Ok(())
    }
} 