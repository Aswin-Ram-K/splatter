//! Window state management.

use serde::{Deserialize, Serialize};

/// Window state for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub label: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
    pub fullscreen: bool,
}

/// Window manager.
pub struct WindowManager {
    states: Vec<WindowState>,
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
        }
    }

    pub fn save(&mut self, state: WindowState) {
        if let Some(existing) = self.states.iter_mut().find(|s| s.label == state.label) {
            *existing = state;
        } else {
            self.states.push(state);
        }
    }

    pub fn get(&self, label: &str) -> Option<&WindowState> {
        self.states.iter().find(|s| s.label == label)
    }

    pub fn all(&self) -> &[WindowState] {
        &self.states
    }
}
