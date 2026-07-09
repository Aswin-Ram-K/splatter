//! # Tauri App
//!
//! Main binary — ties together all splatter-core modules with Tauri commands and events.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent_commands;
mod config_commands;
mod layout_commands;

use splatter_core::{
    agent::AgentManager,
    config,
    crash::CrashReporter,
    layout::LayoutTree,
    notification::NotificationSender,
    plugin::PluginHost,
    tray::TrayManager,
    utils::{app_dirs, AppDirs},
    window::WindowManager,
};
use std::sync::{Arc, Mutex};
use tauri::Manager;

/// Application state shared across Tauri commands.
pub struct AppState {
    pub agents: Arc<Mutex<AgentManager>>,
    pub layout: Arc<Mutex<LayoutTree>>,
    pub config: Arc<Mutex<config::Config>>,
    pub tray: Arc<Mutex<TrayManager>>,
    pub notification: Arc<Mutex<NotificationSender>>,
    pub crash_reporter: Arc<Mutex<CrashReporter>>,
    pub window_manager: Arc<Mutex<WindowManager>>,
    pub plugin_host: Arc<Mutex<PluginHost>>,
    pub dirs: AppDirs,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let dirs = app_dirs();
        dirs.ensure();

        let config = config::load().unwrap_or_else(|_| config::Config::default());

        let agents = AgentManager::new(dirs.data.join("profiles"));
        let layout = LayoutTree::new();
        let tray = TrayManager::new();
        let notification = NotificationSender::new(config.settings.notifications.clone());
        let crash_reporter = CrashReporter::new(
            config.settings.crash_reporting.enabled,
            config.settings.crash_reporting.dsn.clone(),
        );
        let window_manager = WindowManager::new();
        let plugin_host = PluginHost::new(dirs.data.join("plugins"));

        Self {
            agents: Arc::new(Mutex::new(agents)),
            layout: Arc::new(Mutex::new(layout)),
            config: Arc::new(Mutex::new(config)),
            tray: Arc::new(Mutex::new(tray)),
            notification: Arc::new(Mutex::new(notification)),
            crash_reporter: Arc::new(Mutex::new(crash_reporter)),
            window_manager: Arc::new(Mutex::new(window_manager)),
            plugin_host: Arc::new(Mutex::new(plugin_host)),
            dirs,
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let state = AppState::new();

            // Store state in Tauri managed state
            app.manage(state.agents.clone());
            app.manage(state.layout.clone());
            app.manage(state.config.clone());
            app.manage(state.tray.clone());
            app.manage(state.notification.clone());
            app.manage(state.crash_reporter.clone());
            app.manage(state.window_manager.clone());
            app.manage(state.plugin_host.clone());

            // Load agent profiles
            {
                let mut agents = state.agents.lock().unwrap();
                let _ = agents.load_profiles();
            }

            // Load plugins
            {
                let mut plugins = state.plugin_host.lock().unwrap();
                let _ = plugins.load_all();
            }

            // Set up logging
            let _ = env_logger::Builder::from_env(env_logger::Env::default())
                .try_init();

            // Start PTY read loop background task
            let agents = state.agents.clone();
            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                use std::time::Duration;
                loop {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    let mut agents = agents.lock().unwrap();
                    let outputs = agents.drain_outputs();
                    for (agent_id, data) in outputs {
                        let event = splatter_core::agent::AgentOutputEvent {
                            agent_id: agent_id.to_string(),
                            data,
                        };
                        let _ = app_handle.emit("agent-output", event);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Agent commands
            agent_commands::spawn_agent,
            agent_commands::write_to_agent,
            agent_commands::get_agent_state,
            agent_commands::list_agents,
            agent_commands::list_profiles,
            agent_commands::interrupt_agent,
            agent_commands::add_note,
            agent_commands::pin_agent,
            agent_commands::unpin_agent,
            // Layout commands
            layout_commands::new_pane,
            layout_commands::split_pane,
            layout_commands::close_pane,
            layout_commands::focus_direction,
            layout_commands::get_layout,
            layout_commands::set_preset,
            // Config commands
            config_commands::get_config,
            config_commands::save_config,
            config_commands::get_agent_state_for_id,
        ])
        .run(tauri::generate_context!())
        .expect("Failed to run Splatter app");
}
