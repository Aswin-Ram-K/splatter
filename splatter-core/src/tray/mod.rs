//! System tray management.

use serde::{Deserialize, Serialize};

/// System tray status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrayStatus {
    /// No agents running.
    Idle,
    /// One or more agents running.
    Active {
        working: u32,
        done: u32,
        blocked: u32,
        error: u32,
    },
}

impl TrayStatus {
    /// Get the display icon color for this status.
    pub fn icon_color(&self) -> &str {
        match self {
            TrayStatus::Idle => "#888888",
            TrayStatus::Active { error, .. } if *error > 0 => "#ff0000",
            TrayStatus::Active { blocked, .. } if *blocked > 0 => "#ffaa00",
            TrayStatus::Active { .. } => "#00cc00",
        }
    }

    /// Get tooltip text.
    pub fn tooltip(&self) -> String {
        match self {
            TrayStatus::Idle => "Splatter: 0 agents".to_string(),
            TrayStatus::Active {
                working,
                done,
                blocked,
                error,
            } => {
                format!(
                    "Splatter: {} working · {} done · {} blocked · {} error",
                    working, done, blocked, error
                )
            }
        }
    }
}

/// System tray manager.
pub struct TrayManager {
    status: TrayStatus,
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TrayManager {
    pub fn new() -> Self {
        Self {
            status: TrayStatus::Idle,
        }
    }

    pub fn update_status(&mut self, status: TrayStatus) {
        self.status = status;
    }

    pub fn status(&self) -> TrayStatus {
        self.status
    }

    pub fn tick(&mut self, agent_states: &[AgentStateSummary]) {
        let mut working = 0u32;
        let mut done = 0u32;
        let mut blocked = 0u32;
        let mut error = 0u32;

        for state in agent_states {
            match state.status {
                crate::agent::AgentStatus::Working => working += 1,
                crate::agent::AgentStatus::Done => done += 1,
                crate::agent::AgentStatus::Blocked => blocked += 1,
                crate::agent::AgentStatus::Error => error += 1,
                _ => {}
            }
        }

        if working > 0 || done > 0 || blocked > 0 || error > 0 {
            self.update_status(TrayStatus::Active {
                working,
                done,
                blocked,
                error,
            });
        } else {
            self.update_status(TrayStatus::Idle);
        }
    }
}

/// Simple agent state summary for tray updates.
#[derive(Debug, Clone)]
pub struct AgentStateSummary {
    pub status: crate::agent::AgentStatus,
}
