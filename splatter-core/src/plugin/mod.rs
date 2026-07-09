//! Plugin host.
//!
//! Loads, executes, and manages sandboxed plugins.
//! Plugins are JavaScript files with a YAML manifest.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Plugin manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub entry: String, // JS file path relative to plugin dir
    pub permissions: Vec<String>,
    pub api: String, // API version (e.g., "1.0.0")
    pub scripts: PluginScripts,
}

/// Plugin lifecycle scripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginScripts {
    pub on_ready: bool,
    pub on_agent_status_changed: bool,
    pub on_agent_output: bool,
    pub on_hotkey: bool,
}

/// Loaded plugin instance.
#[derive(Debug, Clone)]
pub struct PluginInstance {
    pub manifest: PluginManifest,
    pub enabled: bool,
    pub state: PluginState,
}

/// Plugin execution state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginState {
    Loading,
    Ready,
    Error(String),
}

/// Plugin host.
pub struct PluginHost {
    plugins: HashMap<String, PluginInstance>,
    plugins_dir: PathBuf,
}

impl PluginHost {
    /// Create a new plugin host.
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            plugins_dir,
        }
    }

    /// Load all plugins from the plugins directory.
    pub fn load_all(&mut self) -> Result<(), String> {
        if !self.plugins_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.plugins_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                if let Err(e) = self.load_plugin(&path) {
                    log::warn!("Failed to load plugin {:?}: {}", path, e);
                }
            }
        }
        Ok(())
    }

    /// Load a single plugin from a directory.
    fn load_plugin(&mut self, plugin_path: &PathBuf) -> Result<(), String> {
        let manifest_path = plugin_path.join("plugin.yaml");
        if !manifest_path.exists() {
            return Err("No plugin.yaml found".to_string());
        }

        let manifest_str = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        let manifest: PluginManifest = serde_yaml::from_str(&manifest_str)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;

        let instance = PluginInstance {
            manifest,
            enabled: true,
            state: PluginState::Ready,
        };

        self.plugins.insert(
            plugin_path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            instance,
        );

        Ok(())
    }

    /// List all plugin names.
    pub fn list(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Get a plugin by name.
    pub fn get(&self, name: &str) -> Option<&PluginInstance> {
        self.plugins.get(name)
    }

    /// Enable/disable a plugin.
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(instance) = self.plugins.get_mut(name) {
            instance.enabled = enabled;
            instance.state = if enabled {
                PluginState::Ready
            } else {
                PluginState::Error("Disabled".to_string())
            };
            true
        } else {
            false
        }
    }

    /// Notify all plugins of an agent status change.
    pub fn on_agent_status_changed(&self, agent_id: &str, status: &str) {
        for (name, plugin) in &self.plugins {
            if plugin.enabled && plugin.manifest.scripts.on_agent_status_changed {
                log::debug!(
                    "Plugin '{}' called: onAgentStatusChanged({} -> {})",
                    name,
                    agent_id,
                    status
                );
            }
        }
    }
}
