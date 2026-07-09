//! # Splatter Core
//!
//! Core library for Splatter — the standalone agent-aware terminal multiplexer.
//!
//! Provides:
//! - Agent lifecycle management (spawn, read, write, status)
//! - PTY-based terminal sessions
//! - BSP layout tree
//! - Configuration (TOML)
//! - Plugin host
//! - Notification sender
//! - Window state
//! - Crash reporter interface

pub mod agent;
pub mod config;
pub mod crash;
pub mod hotkey;
pub mod layout;
pub mod notification;
pub mod plugin;
pub mod tray;
pub mod utils;
pub mod window;

pub use agent::{AgentId, AgentManager, AgentProfile, AgentStatus, Session};
pub use config::{Config, Settings, TerminalSettings, CrashReportingSettings};
pub use layout::{LayoutNode, LayoutTree, SplitDirection};
pub use notification::NotificationSender;
pub use plugin::{PluginHost, PluginManifest};
pub use tray::{TrayManager, TrayStatus};
pub use utils::app_dirs;
pub use window::WindowState;
