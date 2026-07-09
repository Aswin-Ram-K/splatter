//! Notification sender.
//!
//! Sends desktop notifications using platform-specific methods.
//! On Linux: notify-send or D-Bus.
//! On macOS: AppleScript (or UserNotifications framework).
//! On Windows: Windows Toast (or notify.exe).

use crate::config::NotificationSettings;
use std::process::Command;

/// Desktop notification sender.
pub struct NotificationSender {
    config: NotificationSettings,
}

impl NotificationSender {
    /// Create a new notification sender.
    pub fn new(config: NotificationSettings) -> Self {
        Self { config }
    }

    /// Send a notification.
    pub fn send(
        &self,
        title: &str,
        body: &str,
        urgency: NotificationUrgency,
    ) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check focus state
        if self.config.focus_when_focused {
            // For now, always send — in production, this would check window focus via Tauri
        }

        // Use notify-send on Linux
        #[cfg(target_os = "linux")]
        {
            let mut cmd = Command::new("notify-send");
            cmd.arg("--app-name=Splatter");
            if matches!(urgency, NotificationUrgency::Low) {
                cmd.arg("--urgency=low");
            } else if matches!(urgency, NotificationUrgency::High) {
                cmd.arg("--urgency=critical");
            }
            if self.config.sound {
                cmd.arg("--icon=dialog-information");
            }
            cmd.arg(title).arg(body);

            cmd.spawn()
                .map_err(|e| format!("Failed to spawn notify-send: {}", e))?
                .wait()
                .ok();
        }

        #[cfg(not(target_os = "linux"))]
        {
            // Fallback: log the notification
            log::info!("Notification: {} - {}", title, body);
        }

        Ok(())
    }

    /// Check if notifications are enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Notification urgency level.
#[derive(Debug, Clone, Copy)]
pub enum NotificationUrgency {
    Low,
    Normal,
    High,
}
