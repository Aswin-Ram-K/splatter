//! Utility functions.

use std::path::PathBuf;

/// Get the application data directories.
pub fn app_dirs() -> AppDirs {
    if let Some(config_dir) = dirs::config_dir() {
        let splatter = config_dir.join("splatter");
        let data_dir = dirs::data_dir()
            .map(|d| d.join("splatter"))
            .unwrap_or_else(|| config_dir.join("splatter"));

        AppDirs {
            config: splatter.clone(),
            data: data_dir,
            cache: dirs::cache_dir()
                .map(|d| d.join("splatter"))
                .unwrap_or_else(|| config_dir.join("splatter").join("cache")),
        }
    } else {
        let local = PathBuf::from(".").join("splatter");
        AppDirs {
            config: local.clone(),
            data: local.clone(),
            cache: local.join("cache"),
        }
    }
}

/// Application directory paths.
pub struct AppDirs {
    pub config: PathBuf,
    pub data: PathBuf,
    pub cache: PathBuf,
}

impl AppDirs {
    /// Ensure all directories exist.
    pub fn ensure(&self) {
        let _ = std::fs::create_dir_all(&self.config);
        let _ = std::fs::create_dir_all(&self.data);
        let _ = std::fs::create_dir_all(&self.cache);
    }
}

/// Format a duration as a human-readable string.
pub fn format_duration(dur: std::time::Duration) -> String {
    let secs = dur.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Format bytes as a human-readable string.
pub fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
