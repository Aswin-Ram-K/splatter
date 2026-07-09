use tauri::Manager;
use splatter_core::agent::{AgentId, AgentProfile, AgentManager};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

/// Spawn a new agent from a profile.
#[tauri::command]
pub async fn spawn_agent(
    app: tauri::AppHandle,
    profile_id: String,
    cols: u16,
    rows: u16,
) -> Result<String, String> {
    let agents = app.state::<Arc<Mutex<AgentManager>>>().inner();
    let mut agents_guard = agents.lock().map_err(|e| e.to_string())?;

    let id = agents_guard.spawn(&profile_id, cols, rows)
        .map_err(|e| e.to_string())?;

    // Emit agent-spawned event to webview
    app.emit("agent-spawned", &id.to_string())
        .map_err(|e| e.to_string())?;

    Ok(id.to_string())
}

/// Write data to an agent's PTY.
#[tauri::command]
pub async fn write_to_agent(
    app: tauri::AppHandle,
    agent_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    let agent_id: AgentId = agent_id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let agents = app.state::<Arc<Mutex<AgentManager>>>().inner();
    let mut agents_guard = agents.lock().map_err(|e| e.to_string())?;

    agents_guard.write(agent_id, &data).map_err(|e| e.to_string())
}

/// Get an agent's state.
#[tauri::command]
pub async fn get_agent_state(
    app: tauri::AppHandle,
    agent_id: String,
) -> Result<serde_json::Value, String> {
    let agent_id: AgentId = agent_id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let agents = app.state::<Arc<Mutex<AgentManager>>>().inner();
    let agents_guard = agents.lock().map_err(|e| e.to_string())?;

    let state = agents_guard.get_state(agent_id)
        .ok_or("Agent not found".to_string())?;

    serde_json::to_value(&state)
        .map_err(|e| e.to_string())
}

/// List all agent IDs.
#[tauri::command]
pub async fn list_agents(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let agents = app.state::<Arc<Mutex<AgentManager>>>().inner();
    let agents_guard = agents.lock().map_err(|e| e.to_string())?;

    let ids = agents_guard.list_agents();
    Ok(ids.iter().map(|id| id.to_string()).collect())
}

/// List all available agent profiles.
#[tauri::command]
pub async fn list_profiles(app: tauri::AppHandle) -> Result<Vec<AgentProfile>, String> {
    let agents = app.state::<Arc<Mutex<AgentManager>>>().inner();
    let agents_guard = agents.lock().map_err(|e| e.to_string())?;

    let profile_ids = agents_guard.list_profiles();
    let profiles: Vec<AgentProfile> = profile_ids
        .iter()
        .filter_map(|id| agents_guard.get_profile(id).cloned())
        .collect();

    Ok(profiles)
}

/// Send SIGINT to an agent.
#[tauri::command]
pub async fn interrupt_agent(
    app: tauri::AppHandle,
    agent_id: String,
) -> Result<(), String> {
    let agent_id: AgentId = agent_id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let agents = app.state::<Arc<Mutex<AgentManager>>>().inner();
    let agents_guard = agents.lock().map_err(|e| e.to_string())?;

    agents_guard.signal(agent_id, nix::sys::signal::Signal::SIGINT)
        .map_err(|e| e.to_string())
}

/// Add a note to an agent.
#[tauri::command]
pub async fn add_note(
    app: tauri::AppHandle,
    agent_id: String,
    note: String,
) -> Result<(), String> {
    let agent_id: AgentId = agent_id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let agents = app.state::<Arc<Mutex<AgentManager>>>().inner();
    let mut agents_guard = agents.lock().map_err(|e| e.to_string())?;

    agents_guard.add_note(agent_id, note);
    Ok(())
}

/// Pin an agent.
#[tauri::command]
pub async fn pin_agent(
    app: tauri::AppHandle,
    agent_id: String,
) -> Result<(), String> {
    let agent_id: AgentId = agent_id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let agents = app.state::<Arc<Mutex<AgentManager>>>().inner();
    let mut agents_guard = agents.lock().map_err(|e| e.to_string())?;

    agents_guard.pin_agent(agent_id);
    Ok(())
}

/// Unpin an agent.
#[tauri::command]
pub async fn unpin_agent(
    app: tauri::AppHandle,
    agent_id: String,
) -> Result<(), String> {
    let agent_id: AgentId = agent_id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let agents = app.state::<Arc<Mutex<AgentManager>>>().inner();
    let mut agents_guard = agents.lock().map_err(|e| e.to_string())?;

    agents_guard.unpin_agent(agent_id);
    Ok(())
}
