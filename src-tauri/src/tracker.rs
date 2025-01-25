use active_win_pos_rs::get_active_window;
use anyhow::Error as AnyhowError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time;
use tracing::{debug, error, info};
use device_query::{DeviceQuery, DeviceState};

use crate::database::{self, DbConnection};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowActivity {
    pub title: String,
    pub application: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_browser: bool,
    pub url: Option<String>,
    pub is_idle: bool,
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
    last_activity: DateTime<Utc>,
    device_state: DeviceState,
    idle_threshold: Duration,
    last_mouse_position: (i32, i32),
}

impl ActivityTracker {
    pub async fn new(db: DbConnection) -> Self {
        Self {
            db,
            current_window: None,
            last_activity: Utc::now(),
            device_state: DeviceState::new(),
            idle_threshold: Duration::from_secs(180), // 3 minutes default
            last_mouse_position: (0, 0),
        }
    }

    pub fn set_idle_threshold(&mut self, seconds: u64) {
        self.idle_threshold = Duration::from_secs(seconds);
    }

    fn check_activity(&mut self) -> bool {
        let current_mouse = self.device_state.get_mouse().coords;
        let keyboard_pressed = !self.device_state.get_keys().is_empty();
        let mouse_moved = current_mouse != self.last_mouse_position;

        if keyboard_pressed || mouse_moved {
            debug!(
                "Activity detected - Mouse: {:?}, Keyboard: {}, Previous Mouse: {:?}",
                current_mouse,
                keyboard_pressed,
                self.last_mouse_position
            );
            self.last_activity = Utc::now();
            self.last_mouse_position = current_mouse;
            true
        } else {
            let idle_duration = Utc::now()
                .signed_duration_since(self.last_activity)
                .to_std()
                .unwrap_or(Duration::from_secs(0));
            
            let is_active = idle_duration < self.idle_threshold;
            debug!(
                "Checking idle - Duration: {:.1?}, Threshold: {:.1?}, Is Active: {}, Mouse: {:?}",
                idle_duration,
                self.idle_threshold,
                is_active,
                current_mouse
            );
            
            if !is_active {
                info!(
                    "🔍 IDLE DETECTED - No activity for {:.1?} (threshold: {:.1?})",
                    idle_duration,
                    self.idle_threshold
                );
            }
            is_active
        }
    }

    pub async fn start_tracking(&mut self) {
        info!("Starting activity tracking");
        let mut interval = time::interval(Duration::from_secs(5)); // Check every 5 seconds

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
        let is_active = self.check_activity();
        
        let activity = WindowActivity {
            title: window.title.clone(),
            application: window.app_name.clone(),
            start_time: now,
            end_time: now,
            is_browser: false,
            url: None,
            is_idle: !is_active,
        };

        info!(
            "💻 Window: {} - {} | Active: {} | Idle: {} | Time: {}",
            activity.application,
            activity.title,
            is_active,
            activity.is_idle,
            now.to_rfc3339()
        );

        // Verifica se devemos criar uma nova atividade ou atualizar a existente
        if let Some(current) = &self.current_window {
            if current.application == activity.application 
                && current.title == activity.title 
                && current.is_idle == activity.is_idle {
                // Atualiza a atividade existente
                let mut updated = current.clone();
                updated.end_time = now;
                
                info!(
                    "🔄 Updating existing activity: {} - {} (idle: {}) | {} -> {}", 
                    updated.application,
                    updated.title,
                    updated.is_idle,
                    updated.start_time.to_rfc3339(),
                    updated.end_time.to_rfc3339()
                );

                database::merge_activity(&self.db, &updated, 300)
                    .await
                    .map_err(AnyhowError::from)?;
            } else {
                // Cria uma nova atividade
                info!(
                    "➕ Creating new activity: {} - {} (idle: {})",
                    activity.application,
                    activity.title,
                    activity.is_idle
                );
                
                database::merge_activity(&self.db, &activity, 300)
                    .await
                    .map_err(AnyhowError::from)?;
            }
        } else {
            // Primeira atividade
            info!(
                "🆕 First activity: {} - {} (idle: {})",
                activity.application,
                activity.title,
                activity.is_idle
            );
            
            database::merge_activity(&self.db, &activity, 300)
                .await
                .map_err(AnyhowError::from)?;
        }
        
        self.current_window = Some(activity);
        Ok(())
    }
} 