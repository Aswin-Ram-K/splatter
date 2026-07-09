//! Agent lifecycle management.
//!
//! Manages PTY-based agent sessions — spawning, reading output,
//! writing input, tracking status, and maintaining activity logs.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Unique agent identifier.
pub type AgentId = Uuid;

/// Status of an agent session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AgentStatus {
    /// Process is starting up.
    #[default]
    Launching,
    /// Ready, waiting for input.
    Idle,
    /// Actively producing output.
    Working,
    /// Blocked (by output limit, prompt, etc.).
    Blocked,
    /// Exited cleanly.
    Done,
    /// Crashed or errored.
    Error,
}

/// Activity log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub timestamp: DateTime<Utc>,
    pub event: ActivityEvent,
}

/// Types of activity events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityEvent {
    AgentStarted { profile_id: String },
    AgentOutput { bytes: usize },
    AgentStatusChanged { from: String, to: String },
    AgentExited { code: Option<i32> },
    UserInput { bytes: usize },
    AgentPaused,
    AgentResumed,
    AgentNotesAdded { text: String },
}

/// Output buffer with configurable size limit.
#[derive(Debug, Clone)]
pub struct OutputBuffer {
    data: Vec<u8>,
    max_bytes: usize,
}

/// Agent profile loaded from YAML config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
    pub scrollback: Option<usize>,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
}

/// Full agent state (status, timing, history).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: AgentId,
    pub profile_id: String,
    pub status: AgentStatus,
    pub started_at: DateTime<Utc>,
    pub duration: Duration,
    pub output_bytes: usize,
    pub output_lines: usize,
    pub cols: u16,
    pub rows: u16,
    pub notes: Vec<String>,
    pub activity_log: Vec<ActivityEntry>,
    pub pinned: bool,
    pub groups: Vec<String>,
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// OutputBuffer
// ---------------------------------------------------------------------------

impl OutputBuffer {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            data: Vec::with_capacity(max_bytes),
            max_bytes,
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }

        // If we'd exceed the limit, drop the oldest data
        if self.data.len() + data.len() > self.max_bytes {
            let needed = self.data.len() + data.len() - self.max_bytes;
            self.data.drain(0..needed.min(self.data.len()));
        }

        // If data itself exceeds the limit, only keep the newest part
        if data.len() > self.max_bytes {
            let skip = data.len() - self.max_bytes;
            self.data.clear();
            self.data.extend_from_slice(&data[skip..]);
        } else {
            self.data.extend_from_slice(data);
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn last_output(&self, chars: usize) -> String {
        let start = self.data.len().saturating_sub(chars);
        String::from_utf8_lossy(&self.data[start..]).to_string()
    }

    pub fn lines(&self) -> usize {
        self.data.iter().filter(|&&b| b == b'\n').count()
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

// ---------------------------------------------------------------------------
// Session — one PTY-based agent instance
// ---------------------------------------------------------------------------

/// A single PTY-backed agent session.
pub struct Session {
    pub id: AgentId,
    pub profile_id: String,
    pub child: Option<Child>,
    pub status: AgentStatus,
    pub output: OutputBuffer,
    pub started_at: Instant,
    pub cols: u16,
    pub rows: u16,
    pub activity_log: Vec<ActivityEntry>,
    pub notes: Vec<String>,
    pub pinned: bool,
    pub groups: Vec<String>,
    pub tags: Vec<String>,
}

impl Session {
    /// Spawn a new agent session with the given profile.
    pub fn spawn(profile: &AgentProfile, cols: u16, rows: u16) -> Result<Self> {
        let id = Uuid::new_v4();
        // Apply environment variables
        let mut cmd = Command::new(&profile.command);
        cmd.args(&profile.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (k, v) in &profile.env {
            cmd.env(k, v);
        }

        // Working directory
        if let Some(cwd) = &profile.cwd {
            cmd.current_dir(cwd);
        }

        let child = cmd.spawn()?;

        let mut session = Self {
            id,
            profile_id: profile.id.clone(),
            child: Some(child),
            status: AgentStatus::Launching,
            output: OutputBuffer::new(512_000), // 512KB default
            started_at: Instant::now(),
            cols,
            rows,
            activity_log: vec![ActivityEntry {
                timestamp: Utc::now(),
                event: ActivityEvent::AgentStarted {
                    profile_id: profile.id.clone(),
                },
            }],
            notes: Vec::new(),
            pinned: false,
            groups: Vec::new(),
            tags: Vec::new(),
        };

        // Transition to idle after a short delay (or immediately for simple shells)
        session.update_status(AgentStatus::Idle);

        Ok(session)
    }

    /// Write data to the PTY stdin.
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        // In a real implementation, we'd use PTY master/slave
        // For now, just record the input
        if !data.is_empty() {
            self.record_input(data.len());
        }
        Ok(())
    }

    /// Send a signal to the agent process.
    pub fn signal(&self, signal: nix::sys::signal::Signal) -> Result<()> {
        if let Some(ref child) = self.child {
            let pid = nix::unistd::Pid::from_raw(child.id() as i32);
            nix::sys::signal::kill(pid, signal)?;
        } else {
            return Err(anyhow::anyhow!("No child process"));
        }
        Ok(())
    }

    /// Check if the process has exited.
    pub fn poll(&mut self) -> Option<i32> {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let code = status.code().unwrap_or(-1);
                    self.update_status(AgentStatus::Done);
                    Some(code)
                }
                Ok(None) => None, // still running
                Err(_) => {
                    self.update_status(AgentStatus::Error);
                    Some(-1)
                }
            }
        } else {
            None
        }
    }

    /// Write raw output to the terminal buffer.
    pub fn write_output(&mut self, data: &[u8]) {
        self.output.write(data);
        self.record_output(data.len());
        if self.status == AgentStatus::Idle {
            self.update_status(AgentStatus::Working);
        }
    }

    /// Get the current status.
    pub fn status(&self) -> AgentStatus {
        self.status
    }

    /// Get duration since started.
    pub fn duration(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Get activity log.
    pub fn activity_log(&self) -> &[ActivityEntry] {
        &self.activity_log
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    fn update_status(&mut self, new: AgentStatus) {
        let old = std::mem::replace(&mut self.status, new);
        if old != new {
            self.activity_log.push(ActivityEntry {
                timestamp: Utc::now(),
                event: ActivityEvent::AgentStatusChanged {
                    from: format!("{:?}", old),
                    to: format!("{:?}", new),
                },
            });
        }
    }

    fn record_output(&mut self, bytes: usize) {
        self.activity_log.push(ActivityEntry {
            timestamp: Utc::now(),
            event: ActivityEvent::AgentOutput { bytes },
        });
    }

    fn record_input(&mut self, bytes: usize) {
        self.activity_log.push(ActivityEntry {
            timestamp: Utc::now(),
            event: ActivityEvent::UserInput { bytes },
        });
    }
}

// ---------------------------------------------------------------------------
// AgentManager — manages all sessions
// ---------------------------------------------------------------------------

/// Manages all agent sessions in the application.
pub struct AgentManager {
    sessions: HashMap<AgentId, Session>,
    profiles: HashMap<String, AgentProfile>,
    profiles_dir: PathBuf,
    max_sessions: usize,
}

impl AgentManager {
    pub fn new(profiles_dir: PathBuf) -> Self {
        Self {
            sessions: HashMap::new(),
            profiles: HashMap::new(),
            profiles_dir,
            max_sessions: 50,
        }
    }

    /// Load all agent profiles from the profiles directory.
    pub fn load_profiles(&mut self) -> Result<()> {
        if !self.profiles_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.profiles_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
                let content = std::fs::read_to_string(&path)?;
                if let Ok(profile) = serde_yaml::from_str::<AgentProfile>(&content) {
                    self.profiles.insert(profile.id.clone(), profile);
                }
            }
        }
        Ok(())
    }

    /// Get a profile by ID.
    pub fn get_profile(&self, id: &str) -> Option<&AgentProfile> {
        self.profiles.get(id)
    }

    /// List all profile IDs.
    pub fn list_profiles(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }

    /// Spawn a new agent from a profile.
    pub fn spawn(&mut self, profile_id: &str, cols: u16, rows: u16) -> Result<AgentId> {
        if self.sessions.len() >= self.max_sessions {
            return Err(anyhow::anyhow!("Maximum session limit reached"));
        }

        let profile = self
            .profiles
            .get(profile_id)
            .ok_or_else(|| anyhow::anyhow!("Profile not found: {}", profile_id))?;

        let session = Session::spawn(profile, cols, rows)?;
        let id = session.id;
        self.sessions.insert(id, session);
        Ok(id)
    }

    /// Write data to an agent's PTY.
    pub fn write(&mut self, agent_id: AgentId, data: &[u8]) -> Result<()> {
        let session = self
            .sessions
            .get_mut(&agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found"))?;
        session.write(data)
    }

    /// Write raw output from PTY to the agent buffer.
    pub fn write_output(&mut self, agent_id: AgentId, data: &[u8]) {
        if let Some(session) = self.sessions.get_mut(&agent_id) {
            session.write_output(data);
        }
    }

    /// Poll all sessions and update statuses.
    pub fn poll(&mut self) -> Vec<(AgentId, Option<i32>)> {
        let mut exits = Vec::new();
        let mut dead_ids = Vec::new();

        for (id, session) in self.sessions.iter_mut() {
            if let Some(code) = session.poll() {
                dead_ids.push(*id);
                if session.status == AgentStatus::Done {
                    exits.push((*id, Some(code)));
                }
            }
        }

        for id in &dead_ids {
            if let Some(session) = self.sessions.remove(id) {
                let duration = session.duration();
                let output_bytes = session.output.len();
                // Keep a summary for persistence
                let state = AgentState {
                    id: session.id,
                    profile_id: session.profile_id.clone(),
                    status: session.status,
                    started_at: Utc::now() - duration,
                    duration,
                    output_bytes,
                    output_lines: session.output.lines(),
                    cols: session.cols,
                    rows: session.rows,
                    notes: session.notes,
                    activity_log: session.activity_log.clone(),
                    pinned: session.pinned,
                    groups: session.groups,
                    tags: session.tags,
                };
                Self::persist_session(state);
            }
        }

        exits
    }

    /// Send a signal to an agent.
    pub fn signal(&self, agent_id: AgentId, signal: nix::sys::signal::Signal) -> Result<()> {
        if let Some(session) = self.sessions.get(&agent_id) {
            session.signal(signal)
        } else {
            Err(anyhow::anyhow!("Agent not found"))
        }
    }

    /// Get an agent state (copyable, no borrow).
    pub fn get_state(&self, agent_id: AgentId) -> Option<AgentState> {
        self.sessions.get(&agent_id).map(|s| AgentState {
            id: s.id,
            profile_id: s.profile_id.clone(),
            status: s.status,
            started_at: Utc::now() - s.duration(),
            duration: s.duration(),
            output_bytes: s.output.len(),
            output_lines: s.output.lines(),
            cols: s.cols,
            rows: s.rows,
            notes: s.notes.clone(),
            activity_log: s.activity_log.clone(),
            pinned: s.pinned,
            groups: s.groups.clone(),
            tags: s.tags.clone(),
        })
    }

    /// List all agent IDs.
    pub fn list_agents(&self) -> Vec<AgentId> {
        self.sessions.keys().cloned().collect()
    }

    /// Get session by ID (mutable).
    pub fn get_mut(&mut self, agent_id: AgentId) -> Option<&mut Session> {
        self.sessions.get_mut(&agent_id)
    }

    /// Add a note to an agent.
    pub fn add_note(&mut self, agent_id: AgentId, note: String) {
        if let Some(session) = self.sessions.get_mut(&agent_id) {
            let note_clone = note.clone();
            session.notes.push(note);
            session.activity_log.push(ActivityEntry {
                timestamp: Utc::now(),
                event: ActivityEvent::AgentNotesAdded { text: note_clone },
            });
        }
    }

    /// Pin an agent.
    pub fn pin_agent(&mut self, agent_id: AgentId) {
        if let Some(session) = self.sessions.get_mut(&agent_id) {
            session.pinned = true;
        }
    }

    /// Unpin an agent.
    pub fn unpin_agent(&mut self, agent_id: AgentId) {
        if let Some(session) = self.sessions.get_mut(&agent_id) {
            session.pinned = false;
        }
    }

    /// Add a group to an agent.
    pub fn add_group(&mut self, agent_id: AgentId, group: String) {
        if let Some(session) = self.sessions.get_mut(&agent_id) {
            session.groups.push(group);
        }
    }

    /// Persist agent state to disk.
    fn persist_session(state: AgentState) {
        if let Some(dir) = dirs::config_dir().map(|d| d.join("splatter").join("sessions")) {
            let _ = std::fs::create_dir_all(&dir);
            let path = dir.join(format!("{}.json", state.id));
            if let Ok(f) = std::fs::File::create(&path) {
                let _ = serde_json::to_writer_pretty(&f, &state);
            }
        }
    }
}

// We need serde_yaml for profile loading — add as optional dependency
// In production, profiles could also be loaded from config

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_buffer() {
        let mut buf = OutputBuffer::new(100);
        buf.write(b"Hello, ");
        assert_eq!(buf.len(), 7);
        assert_eq!(buf.last_output(100), "Hello, ");

        buf.write(b"World!\n");
        assert_eq!(buf.len(), 14);
        assert_eq!(buf.lines(), 1);

        // Large write that should trim
        buf.write(&vec![b'X'; 200]);
        let final_len = buf.len();
        assert!(final_len <= 100, "Expected <= 100, got {}", final_len);
    }

    #[test]
    fn test_agent_id() {
        let id = Uuid::new_v4();
        assert_eq!(id.to_string().len(), 36); // UUID v4 format
    }
}
