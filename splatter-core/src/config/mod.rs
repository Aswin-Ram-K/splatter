//! Configuration management.
//!
//! Loads and persists `config.toml` with support for migration.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub settings: Settings,
}

/// User-facing settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub terminal: TerminalSettings,
    pub agents: AgentsSettings,
    pub notifications: NotificationSettings,
    pub hotkeys: HotkeySettings,
    pub crash_reporting: CrashReportingSettings,
}

/// Terminal settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSettings {
    pub font_family: String,
    pub font_size: f64,
    pub scrollback: usize,
    pub theme: String,
    pub cursor_style: String,
    pub mouse_tracking: bool,
    pub bracketed_paste: bool,
    pub input_batch_delay_ms: u64,
}

/// Agent settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsSettings {
    pub max_sessions: usize,
    pub output_buffer_mb: usize,
    pub auto_focus_on_spawn: bool,
    pub show_agent_list: bool,
}

/// Notification settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub sound: bool,
    pub focus_when_focused: bool,
    pub coalesce_window_seconds: u64,
    pub triggers: Vec<String>,
}

/// Hotkey settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeySettings {
    pub nav_prev_pane: String,
    pub nav_next_pane: String,
    pub nav_cycle_pane: String,
    pub nav_focus_left: String,
    pub nav_focus_down: String,
    pub nav_focus_up: String,
    pub nav_focus_right: String,
    pub nav_split_right: String,
    pub nav_split_down: String,
    pub nav_zoom_toggle: String,
    pub nav_close_pane: String,
    pub agent_interrupt: String,
    pub agent_new: String,
    pub window_new: String,
}

/// Crash reporting settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReportingSettings {
    pub enabled: bool,
    pub dsn: String,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            font_family: "JetBrains Mono".to_string(),
            font_size: 15.0,
            scrollback: 10000,
            theme: "dark".to_string(),
            cursor_style: "block".to_string(),
            mouse_tracking: true,
            bracketed_paste: true,
            input_batch_delay_ms: 50,
        }
    }
}

impl Default for AgentsSettings {
    fn default() -> Self {
        Self {
            max_sessions: 50,
            output_buffer_mb: 512,
            auto_focus_on_spawn: true,
            show_agent_list: true,
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            sound: true,
            focus_when_focused: true,
            coalesce_window_seconds: 30,
            triggers: vec![
                "agent_blocked".to_string(),
                "agent_done".to_string(),
                "agent_error".to_string(),
            ],
        }
    }
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            nav_prev_pane: "Ctrl+PageUp".to_string(),
            nav_next_pane: "Ctrl+PageDown".to_string(),
            nav_cycle_pane: "Ctrl+Tab".to_string(),
            nav_focus_left: "Ctrl+h".to_string(),
            nav_focus_down: "Ctrl+j".to_string(),
            nav_focus_up: "Ctrl+k".to_string(),
            nav_focus_right: "Ctrl+l".to_string(),
            nav_split_right: "Ctrl+Shift+e".to_string(),
            nav_split_down: "Ctrl+Shift+o".to_string(),
            nav_zoom_toggle: "Ctrl+z".to_string(),
            nav_close_pane: "Ctrl+d".to_string(),
            agent_interrupt: "Ctrl+c".to_string(),
            agent_new: "Ctrl+n".to_string(),
            window_new: "Ctrl+Shift+n".to_string(),
        }
    }
}

impl Default for CrashReportingSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            dsn: "".to_string(),
        }
    }
}

/// Configuration file path.
pub fn config_path() -> PathBuf {
    if let Some(dir) = dirs::config_dir() {
        dir.join("splatter").join("config.toml")
    } else {
        PathBuf::from("./config.toml")
    }
}

/// Profile directory for agent YAML files.
pub fn profiles_dir() -> PathBuf {
    if let Some(dir) = dirs::config_dir() {
        dir.join("splatter").join("profiles")
    } else {
        PathBuf::from("./profiles")
    }
}

/// Load configuration from disk.
pub fn load() -> Result<Config, String> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;

    let config: Config =
        toml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;

    Ok(config)
}

/// Save configuration to disk.
pub fn save(config: &Config) -> Result<(), String> {
    let dir = config_path()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&dir).ok();

    let content =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(config_path(), content).map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.settings.terminal.font_size, 15.0);
        assert_eq!(config.settings.terminal.scrollback, 10000);
        assert_eq!(config.settings.agents.max_sessions, 50);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.settings.terminal.font_size, 15.0);
    }
}
