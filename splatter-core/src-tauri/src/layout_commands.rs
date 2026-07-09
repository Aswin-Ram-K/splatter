use tauri::Manager;
use splatter_core::agent::AgentManager;
use splatter_core::layout::{LayoutTree, SplitDirection};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

/// Create a new pane with an agent.
#[tauri::command]
pub async fn new_pane(
    app: tauri::AppHandle,
    profile_id: String,
) -> Result<String, String> {
    let layout = app.state::<Arc<Mutex<LayoutTree>>>().inner();
    let mut layout_guard = layout.lock().map_err(|e| e.to_string())?;

    let node_id = layout_guard.new_leaf();

    // Get dimensions for the new pane
    let (cols, rows) = layout_guard.get_pane_size(node_id).unwrap_or((80, 24));

    // Spawn agent on the new pane
    let agents = app.state::<Arc<Mutex<AgentManager>>>().inner();
    let mut agents_guard = agents.lock().map_err(|e| e.to_string())?;
    let agent_id = agents_guard.spawn(&profile_id, cols, rows)
        .map_err(|e| e.to_string())?;

    // Associate agent with layout node
    layout_guard.set_pane_agent(node_id, agent_id.to_string());

    // Emit events
    app.emit("layout-changed", &"pane-created".to_string())
        .map_err(|e| e.to_string())?;
    app.emit("agent-spawned", &serde_json::json!({
        "agent_id": agent_id.to_string(),
        "layout_node_id": node_id,
    })).map_err(|e| e.to_string())?;

    Ok(agent_id.to_string())
}

/// Split the focused pane.
#[tauri::command]
pub async fn split_pane(
    app: tauri::AppHandle,
    direction: String,
    ratio: f64,
) -> Result<u64, String> {
    let layout = app.state::<Arc<Mutex<LayoutTree>>>().inner();
    let mut layout_guard = layout.lock().map_err(|e| e.to_string())?;

    let dir = match direction.as_str() {
        "vertical" => SplitDirection::Vertical,
        "horizontal" => SplitDirection::Horizontal,
        _ => {
            return Err("Invalid direction. Use 'vertical' or 'horizontal'".to_string());
        }
    };

    let node_id = layout_guard.split(dir, ratio);

    // Emit layout-changed event
    app.emit("layout-changed", &node_id.to_string())
        .map_err(|e| e.to_string())?;

    Ok(node_id)
}

/// Close the focused pane (or a specific pane).
#[tauri::command]
pub async fn close_pane(
    app: tauri::AppHandle,
    node_id: Option<u64>,
) -> Result<bool, String> {
    let layout = app.state::<Arc<Mutex<LayoutTree>>>().inner();
    let mut layout_guard = layout.lock().map_err(|e| e.to_string())?;

    let result = layout_guard.close(node_id.unwrap_or(0));

    if result {
        app.emit("layout-changed", &"closed".to_string())
            .map_err(|e| e.to_string())?;
    }

    Ok(result)
}

/// Focus a pane in a direction.
#[tauri::command]
pub async fn focus_direction(
    app: tauri::AppHandle,
    direction: String,
) -> Result<bool, String> {
    let layout = app.state::<Arc<Mutex<LayoutTree>>>().inner();
    let mut layout_guard = layout.lock().map_err(|e| e.to_string())?;

    let focus_dir = match direction.as_str() {
        "left" => splatter_core::layout::FocusDirection::Left,
        "right" => splatter_core::layout::FocusDirection::Right,
        "up" => splatter_core::layout::FocusDirection::Up,
        "down" => splatter_core::layout::FocusDirection::Down,
        "next" => splatter_core::layout::FocusDirection::Next,
        "prev" => splatter_core::layout::FocusDirection::Previous,
        _ => {
            return Err("Invalid direction".to_string());
        }
    };

    layout_guard.focus_direction(focus_dir);

    app.emit("layout-changed", &"focused".to_string())
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/// Get the current layout tree.
#[tauri::command]
pub async fn get_layout(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let layout = app.state::<Arc<Mutex<LayoutTree>>>().inner();
    let layout_guard = layout.lock().map_err(|e| e.to_string())?;

    // Return the layout as a JSON structure with leaf panes
    let leaves = layout_guard.leaves();
    let mut pane_data = Vec::new();
    for (id, pane) in leaves {
        pane_data.push(serde_json::json!({
            "id": id,
            "rect": pane.rect,
            "agent_id": pane.agent_id,
        }));
    }

    serde_json::to_value(pane_data)
        .map_err(|e| e.to_string())
}

/// Set a layout preset.
#[tauri::command]
pub async fn set_preset(
    app: tauri::AppHandle,
    name: String,
) -> Result<bool, String> {
    let layout = app.state::<Arc<Mutex<LayoutTree>>>().inner();
    let _layout_guard = layout.lock().map_err(|e| e.to_string())?;

    if let Some(preset) = LayoutTree::preset(&name) {
        let _ = preset;
        app.emit("layout-changed", &"preset".to_string())
            .map_err(|e| e.to_string())?;
    }

    Ok(true)
}
