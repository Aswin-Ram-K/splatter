//! Crash reporting.

/// Crash reporter interface.
pub struct CrashReporter {
    enabled: bool,
    dsn: String,
}

impl CrashReporter {
    pub fn new(enabled: bool, dsn: String) -> Self {
        Self { enabled, dsn }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Report a crash (sentry integration).
    pub fn report(&self, error: &str) {
        if !self.enabled {
            log::debug!("Crash reporting disabled, skipping: {}", error);
            return;
        }
        log::error!("Crash reported: {}", error);
        // In production, this would send to Sentry:
        // sentry::capture_message(error, sentry::Level::Error);
    }

    /// Set the DSN.
    pub fn set_dsn(&mut self, dsn: String) {
        self.dsn = dsn;
    }

    /// Enable/disable crash reporting.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
