use tauri::Manager;
use splatter_core::config;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

/// Get current config.
#[tauri::command]
pub async fn get_config(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let config = app.state::<Arc<Mutex<config::Config>>>().inner();
    let config_guard = config.lock().map_err(|e| e.to_string())?;

    serde_json::to_value(&*config_guard)
        .map_err(|e| e.to_string())
}

/// Save config (persists to disk).
#[tauri::command]
pub async fn save_config(app: tauri::AppHandle, config: config::Config) -> Result<(), String> {
    // Save config to disk
    config::save(&config).map_err(|e| e.to_string())?;

    // Update state
    {
        let state = app.state::<Arc<Mutex<config::Config>>>().inner();
        let mut config_guard = state.lock().map_err(|e| e.to_string())?;
        *config_guard = config;
    }

    app.emit("config-changed", ())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get an agent's state by ID.
#[tauri::command]
pub async fn get_agent_state_for_id(
    app: tauri::AppHandle,
    agent_id: String,
) -> Result<serde_json::Value, String> {
    let agent_id: uuid::Uuid = agent_id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let agents = app.state::<Arc<Mutex<splatter_core::agent::AgentManager>>>().inner();
    let agents_guard = agents.lock().map_err(|e| e.to_string())?;

    let state = agents_guard.get_state(agent_id)
        .ok_or("Agent not found".to_string())?;

    serde_json::to_value(&state)
        .map_err(|e| e.to_string())
}
