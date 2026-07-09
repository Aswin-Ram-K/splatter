use tauri::Manager;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

/// Split the focused pane.
#[tauri::command]
pub async fn split_pane(
    app: tauri::AppHandle,
    direction: String,
    ratio: f64,
) -> Result<u64, String> {
    let layout = app.state::<Arc<Mutex<splatter_core::layout::LayoutTree>>>().inner();
    let mut layout_guard = layout.lock().map_err(|e| e.to_string())?;

    let dir = match direction.as_str() {
        "vertical" => splatter_core::layout::SplitDirection::Vertical,
        "horizontal" => splatter_core::layout::SplitDirection::Horizontal,
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
    let layout = app.state::<Arc<Mutex<splatter_core::layout::LayoutTree>>>().inner();
    let mut layout_guard = layout.lock().map_err(|e| e.to_string())?;

    let node_id = node_id.unwrap_or(layout_guard.focused_id());
    let result = layout_guard.close(node_id);

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
    let layout = app.state::<Arc<Mutex<splatter_core::layout::LayoutTree>>>().inner();
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
    let layout = app.state::<Arc<Mutex<splatter_core::layout::LayoutTree>>>().inner();
    let layout_guard = layout.lock().map_err(|e| e.to_string())?;

    serde_json::to_value(&*layout_guard)
        .map_err(|e| e.to_string())
}

/// Set a layout preset.
#[tauri::command]
pub async fn set_preset(
    app: tauri::AppHandle,
    name: String,
) -> Result<bool, String> {
    let layout = app.state::<Arc<Mutex<splatter_core::layout::LayoutTree>>>().inner();
    let mut layout_guard = layout.lock().map_err(|e| e.to_string())?;

    if let Some(preset) = splatter_core::layout::LayoutTree::preset(&name) {
        *layout_guard = preset;
        app.emit("layout-changed", &"preset".to_string())
            .map_err(|e| e.to_string())?;
    }

    Ok(true)
}
