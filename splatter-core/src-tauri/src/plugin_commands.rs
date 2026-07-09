use tauri::Manager;
use splatter_core::plugin::PluginHost;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

/// List all plugins.
#[tauri::command]
pub async fn list_plugins(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let plugins = app.state::<Arc<Mutex<PluginHost>>>().inner();
    let plugins_guard = plugins.lock().map_err(|e| e.to_string())?;
    
    let list = plugins_guard.list();
    serde_json::to_value(&list)
        .map_err(|e| e.to_string())
}

/// Enable or disable a plugin.
#[tauri::command]
pub async fn toggle_plugin(
    app: tauri::AppHandle,
    name: String,
    enabled: bool,
) -> Result<bool, String> {
    let plugins = app.state::<Arc<Mutex<PluginHost>>>().inner();
    let mut plugins_guard = plugins.lock().map_err(|e| e.to_string())?;
    
    let result = plugins_guard.set_enabled(&name, enabled);
    
    if result && enabled {
        app.emit("plugin-enabled", &name)
            .map_err(|e| e.to_string())?;
    } else if result && !enabled {
        app.emit("plugin-disabled", &name)
            .map_err(|e| e.to_string())?;
    }
    
    Ok(result)
}

/// Get plugin status.
#[tauri::command]
pub async fn get_plugin_status(app: tauri::AppHandle, name: String) -> Result<serde_json::Value, String> {
    let plugins = app.state::<Arc<Mutex<PluginHost>>>().inner();
    let plugins_guard = plugins.lock().map_err(|e| e.to_string())?;
    
    if let Some(plugin) = plugins_guard.get(&name) {
        serde_json::to_value(&serde_json::json!({
            "name": name,
            "enabled": plugin.enabled,
            "state": format!("{:?}", plugin.state),
        }))
        .map_err(|e| e.to_string())
    } else {
        Ok(serde_json::json!({ "error": "Plugin not found" }))
    }
}
