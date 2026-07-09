# Splatter — Standalone Agent-Aware Terminal Multiplexer

## Product Vision

**Splatter** is a standalone, agent-aware terminal multiplexer — a native desktop application
that provides a CMUX-style terminal interface with full agent lifecycle management. It launches
agents into real terminal panes, tracks their state in real time, and provides a complete
agent-aware control surface. No browser, no herdr dependency, no web dashboard — just a native
desktop app.

**Target:** Linux v1 (GTK/WebKitGTK). macOS and Windows follow.

## Decision Log

| # | Decision | Choice |
|---|----------|--------|
| 1 | Product scope | **Standalone meta-Herder** — no herdr dependency, rebuild from scratch |
| 2 | Agent model | **Launcher mode** — Splatter launches agents into panes |
| 3 | Remote sessions | **Local only for v1** — SSH remote later |
| 4 | Agent presets | **Pi agent + master AI stack** — two primary B1 focuses |
| 5 | Terminal rendering | **Ghostty Web in WebView** — proven, zero dev time on VT emulation |
| 6 | UI framework | **React + TypeScript** — same as herdr-web |
| 7 | Multi-window | **Multi-window per screen** — each monitor gets its own window |
| 8 | Agent-aware UI | **Full meta-Herder** — status dots, history, timeline, stats, handoff, notes |
| 9 | System tray | **Full status panel** — colored icon, tooltip with counts, menu with pin/quick actions |
| 10 | Notifications | **Full suite with user control** — all triggers configurable |
| 11 | Global hotkeys | **Navigation + actions + agent** — all shortcuts plus agent control |
| 12 | Layout presets | **Built-in + custom** — ship presets AND let users save/export/import |
| 13 | Agent resume | **Resume + replay** — one-click restart + activity replay |
| 14 | Terminal extras | **Find + history + multi-cursor** — Ctrl+F, configurable scrollback, multi-pane input |
| 15 | Themes | **Dark only** — single dark theme, minimal complexity |
| 16 | Plugin system | **Full plugin marketplace** — sandboxed, manifest, GitHub-hosted marketplace |
| 17 | Auto-updater | **Yes + crash reporting** — Tauri updater + Sentry |
| 18 | Multi-monitor | **Independent windows** — each monitor has its own window, separate or shared session |
| 19 | Settings | **Full UI + import/export** — structured config.toml + in-app panel + migrate |
| 20 | Pinning | **Pin + groups + filters + sort** — full sidebar organization |
| 21 | Platform (v1) | **Linux only** — .deb/.AppImage |
| 22 | CI/CD | **CI + package managers** — GitHub Actions + Homebrew + AUR + winget |

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Splatter (Tauri 2 App)                               │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │  Window 1 (Monitor 1)                              Window 2 (Monitor 2) ││
│  │  ┌─────────────┬──────────────────────────────────────────────────┐    ││
│  │  │ Sidebar     │  Terminal Grid (Pane Grid)                        ││    ││
│  │  │             │                                                   ││    ││
│  │  │ 🟢 Pi A     │  ┌──────────┐  ┌──────────┐  ┌──────────┐       ││    ││
│  │  │ 🟡 Pi B     │  │ Pane 1   │  │ Pane 2   │  │ Pane 3   │       ││    ││
│  │  │ 🔵 Pi C     │  │ (bash)   │  │ (Claude) │  │ (agent)  │       ││    ││
│  │  │ ⚪ Pi D     │  │          │  │          │  │          │       ││    ││
│  │  │             │  └──────────┘  └────┬─────┘  └──────────┘       ││    ││
│  │  │ ──────────  │  ┌──────────────────┴──┐   ┌────────────────┐   ││    ││
│  │  │ Pinned      │  │ Pane 4              │   │ Agent Notes    │   ││    ││
│  │  │ 🟢 Pi E     │  │ (working - 12min)   │   │ (collapsible)  │   ││    ││
│  │  │ 🟡 Pi F     │  └─────────────────────┘   └────────────────┘   ││    ││
│  │  │ ──────────  │                                                   ││    ││
│  │  │ Groups      │  ┌────────────────────────────────────────────┐  ││    ││
│  │  │ 🏠 Project  │  │ Status Bar: 3 working · 2 blocked · 1 idle │  ││    ││
│  │  │ 🏠 Work     │  └────────────────────────────────────────────┘  ││    ││
│  │  │             └─────────────────────────────────────────────────┘    ││
│  │  └───────────────────────────────────────────────────────────────────┘│
│  │                                                                             │
│  └─────────────────────────────────────────────────────────────────────────────┘│
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                    Rust Core (Tauri Commands + State)                   ││
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐  ││
│  │  │ Agent    │ │ Layout   │ │ Tray     │ │ Hotkey   │ │ Plugin     │  ││
│  │  │ Manager  │ │ Engine   │ │ Manager  │ │ Registry │ │ Host       │  ││
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └────────────┘  ││
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐  ││
│  │  │ Notify   │ │ Auto-    │ │ Settings │ │ Crash    │ │ Plugin     │  ││
│  │  │ Engine   │ │ Updater  │ │ Store    │ │ Reporter │ │ Marketplace│  ││
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └────────────┘  ││
│  └─────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
```

## Component Breakdown

### 1. Tauri 2 App Shell

**Tech:** Tauri 2.x (Rust + WebView)

- **Frontend:** React 19 + TypeScript + Vite
- **Backend:** Rust (Tauri commands, Tauri plugins)
- **WebView:** WebKitGTK (Linux v1)
- **Binary:** ~10-20 MB (WebView bundled)

**Key Tauri 2 plugins:**

- `tauri-plugin-shell` — spawn processes (agent launcher)
- `tauri-plugin-global-shortcut` — system-wide hotkeys
- `tauri-plugin-store` — persistent settings
- `tauri-plugin-notification` — desktop notifications
- `tauri-plugin-updater` — auto-updater
- Custom plugin: `tauri-plugin-tray` — system tray management
- Custom plugin: `tauri-plugin-plugin-host` — plugin system

**Linux-specific:**

- GTK4 / WebKitGTK 2.44+
- Wayland and X11 support (Tauri handles both)
- Desktop entry for `.local/share/applications`
- AppIcon for system tray

### 2. Agent Launcher System

Splatter is a launcher — it starts agents in real PTY panes.

**Agent profiles** are defined in `~/.config/splatter/agents/`:

```yaml
# ~/.config/splatter/agents/pi.yaml
name: Pi
display_name: "Pi Agent"
icon: "🤖"
command: "pi"
args: []
env:
  PI_TOKEN: "${env:PI_TOKEN}"
  PI_MODEL: "${env:PI_MODEL}"
cwd: "${workspace}"
working_dir_behavior: inherit
detect_rules:
  - pattern: "pi-agent"
    agent_type: "pi"
    confidence: 0.95
ui_features:
  resume: true
  interrupt: true
  continue: true
  handoff: true
  notes: true
status_mapping:
  idle: "idle"
  working: "working"
  blocked: "blocked"
  done: "done"
  error: "blocked"
```

**Launch flow:**

1. User clicks "New Agent" → agent profile picker
2. User selects "Pi" → Splatter spawns `pi` in a new PTY pane
3. Splatter detects the agent type from launch command + patterns
4. Agent status events flow through the agent manager

**Agent lifecycle:**

- `launching` → `working` → `idle` / `blocked` / `done` / `error`
- Each status transition is logged with timestamp
- Transitions are emitted as events to the UI and notifications engine

### 3. Ghostty Web Terminal Rendering

**Tech:** `ghostty-web` npm package in Tauri WebView

ghostty-web provides a VT100 terminal emulator via Emscripten that runs in JavaScript. It:

- Emulates xterm/VT100 completely (true color, mouse protocol, bracketed paste, OSC 52)
- Renders to canvas (via libghostty via Emscripten)
- Exposes an Xterm-compatible API for input/output

**Integration:**

```typescript
// In the React app — each terminal pane
import { GhosttyTerminal } from "@ghostty-web";

function TerminalPane({ terminalId, cols, rows }: Props) {
  const ghosttyRef = useRef<GhosttyTerminal>(null);
  const inputRef = useRef<HTMLDivElement>(null);

  // Ghostty emits output → forward to Rust via Tauri commands
  useEffect(() => {
    const handler = (data: string) => {
      invoke('terminal_input', { terminalId, data });
    };
    ghosttyRef.current?.on('data', handler);
    return () => ghosttyRef.current?.off('data', handler);
  }, []);

  // Ghostty receives input from Rust → feed to terminal
  useEffect(() => {
    const handler = (data: string) => {
      ghosttyRef.current?.write(data);
    };
    // Listen to Tauri events for terminal output
    window.addEventListener('terminal-output', handler);
    return () => window.removeEventListener('terminal-output', handler);
  }, []);

  return (
    <div className="terminal-container">
      <GhosttyTerminal
        ref={ghosttyRef}
        cols={cols}
        rows={rows}
        fontFamily="JetBrains Mono"
        fontSize={14}
        theme={darkTheme}
        backgroundOpacity={1}
        cursorBlink={true}
        cursorStyle="block"
      />
    </div>
  );
}
```

**Ghostty-web features we use:**

- True color (24-bit)
- Mouse protocol (click to focus, selection)
- Bracketed paste
- OSC 52 (terminal-initiated paste — critical for agent workflows)
- Ligatures
- Window management protocol
- Sixel/lima image display (optional, later)

**Performance:**

- Each pane runs ghostty-web in its own canvas element
- Input/output goes through Tauri IPC (Rust ↔ JS bridge)
- Input batching: 32ms delay for normal typing, immediate for paste
- Output coalescing: batch VT output frames at 60fps

### 4. Multi-Window Manager

Each monitor can have its own Splatter window. Windows can be:

- **Independent** — each has its own session (separate workspaces, panes, agents)
- **Shared** — multiple windows show the same session (like a mirrored view)

**Window management:**

```rust
// In Rust
use tauri::Manager;

struct WindowManager {
    windows: HashMap<String, tauri::Window>,
    // Window → Session mapping
    window_sessions: HashMap<String, Option<String>>, // None = independent, Some = shared
    // Monitor → Window mapping
    monitor_windows: HashMap<String, String>,
}

impl WindowManager {
    // Create a new window on a specific monitor
    fn create_window(&self, monitor_id: &str, session: Option<String>) -> Result<tauri::Window>;

    // Share a session between two windows
    fn link_session(&self, window_a: &str, window_b: &str) -> Result<()>;

    // Unlink a window from its session
    fn unlink_session(&self, window_id: &str) -> Result<()>;

    // Detect monitors and create default layout
    fn detect_and_layout(&self) -> Result<()>;

    // Handle monitor hotplug (new monitor connected/disconnected)
    fn on_monitor_change(&self) -> Result<()>;
}
```

**Window state persistence:**

- Window position, size, z-order per monitor
- Session association (independent/shared)
- Restored on app launch

### 5. Layout Engine (Pane Grid)

Each window manages its own pane grid — a binary space partition (BSP) tree like tmux.

**Layout primitives:**

```rust
struct Split {
    id: String,
    direction: SplitDirection, // Right or Down
    ratio: f32,               // 0.0 - 1.0
    child_a: PaneId,          // Left or Top
    child_b: PaneId,          // Right or Bottom
}

struct Pane {
    id: String,
    terminal_id: String,
    rect: Rect,               // x, y, width, height (relative to pane grid)
    zoomed: bool,
    focused: bool,
}
```

**Operations:**

- `split_right(pane_id) → new_pane_id`
- `split_down(pane_id) → new_pane_id`
- `close_pane(pane_id) → merged_pane_id` (adjacent panes merge)
- `focus_direction(pane_id, direction) → target_pane_id`
- `focus_next(pane_id) → target_pane_id`
- `focus_prev(pane_id) → target_pane_id`
- `zoom_pane(pane_id) → (was_zoomed, target_pane_id)`
- `resize_pane(pane_id, delta) → ()`
- `swap_panes(pane_id_a, pane_id_b) → ()`
- `move_pane(pane_id, direction, new_parent) → ()`

**Layout presets** (built-in + custom):

```yaml
# Built-in presets (ship with app)
presets:
  - name: "2-pane horizontal"
    type: split
    direction: right
    ratio: 0.5
  - name: "2-pane vertical"
    type: split
    direction: down
    ratio: 0.5
  - name: "3-pane row"
    type: split
    direction: right
    ratio: 0.33
    child:
      type: split
      direction: down
      ratio: 0.5
  - name: "2×2 grid"
    type: split
    direction: right
    ratio: 0.5
    child:
      type: split
      direction: down
      ratio: 0.5
  - name: "sidebar + main"
    type: split
    direction: right
    ratio: 0.25
    child:
      type: split
      direction: down
      ratio: 0.75

# Custom presets (user-saved)
# Saved to: ~/.config/splatter/presets/<name>.yaml
```

### 6. Agent Awareness Engine

The core "meta-Herder" capability. Tracks agent lifecycle and provides a rich control surface.

**Agent state model:**

```rust
struct AgentState {
    pane_id: String,
    terminal_id: String,
    profile_name: String,
    display_name: String,
    agent_type: AgentType,    // pi, claude, codex, etc.
    status: AgentStatus,      // idle, working, blocked, done, error, launching
    started_at: Instant,
    last_status_at: Instant,
    last_output_at: Instant,
    duration: Duration,
    pin: bool,
    group: Option<String>,    // workspace/project/group
    tags: Vec<String>,
    notes: Vec<Note>,
    activity_log: Vec<ActivityEntry>,
    performance: AgentStats,
    resume_token: Option<String>,
}

struct ActivityEntry {
    timestamp: Instant,
    event: ActivityEvent,     // status_change, output, interrupted, etc.
    detail: String,           // Human-readable summary
}

struct AgentStats {
    total_time: Duration,
    active_time: Duration,
    idle_time: Duration,
    blocked_time: Duration,
    output_bytes: u64,
    output_lines: u64,
    command_count: u64,
    file_reads: u64,
    file_writes: u64,
    errors: u64,
}
```

**Agent types and detection:**

Each agent type has a detection profile:

```rust
struct AgentProfile {
    name: String,
    display_name: String,
    icon: &'static str,          // Emoji or icon name
    status_colors: StatusColors, // Per-status color
    detection: AgentDetection,   // How to identify this agent running
    capabilities: AgentCapabilities, // What actions are available
    status_map: StatusMap,       // Map raw status → Splatter status
}
```

**Agent capabilities (per type):**

- `resume` — can the agent be resumed? (e.g., Pi agents can be resumed)
- `interrupt` — can the agent be interrupted? (Ctrl+C equivalent)
- `continue` — can the agent continue from where it left off?
- `handoff` — can the agent hand off work to another agent?
- `notes` — can the user annotate this agent's work?
- `activity_replay` — can we show an activity replay?
- `session_export` — can we export the agent's session?

**Agent handoff:**

When agent A finishes and needs to pass work to agent B:

1. User clicks "Handoff" on agent A's completed pane
2. Splatter captures the work state (recent commands, file context)
3. User selects agent B to hand off to
4. Splatter launches agent B with context from agent A

**Agent notes:**

Per-pane, per-agent notes that persist across sessions:

```rust
struct Note {
    id: String,
    agent_id: String,
    title: String,
    body: String,
    created_at: Instant,
    updated_at: Instant,
    tags: Vec<String>,
    linked_agents: Vec<String>,
}
```

### 7. System Tray Manager

Tauri 2 tray with colored status indicator.

**Tray state model:**

```rust
struct TrayManager {
    icon: TrayIcon,
    current_status: TrayStatus, // Global app status
    agent_count: usize,         // Total agents managed
    status_counts: StatusCounts, // { working: 2, blocked: 1, ... }
    tooltip: String,             // "2 working · 1 blocked"
}

fn update_tray(&self) {
    // Set colored icon based on most critical status
    match self.most_critical_status() {
        Some(Status::Blocked) => self.set_icon(red_icon),
        Some(Status::Working) => self.set_icon(green_icon),
        Some(Status::Done) => self.set_icon(gray_icon),
        None => self.set_icon(default_icon),
    }

    // Set tooltip with summary
    self.set_tooltip(format!(
        "{} working · {} blocked · {} idle",
        self.status_counts.working,
        self.status_counts.blocked,
        self.status_counts.idle
    ));
}
```

**Tray menu items:**

```rust
// Tray menu (right-click)
- Show Splatter
- ──────────────────
- Recent Agent Status:
  - 🟢 Pi Agent A (working - 2m)
  - 🟡 Pi Agent B (blocked - 5m)
  - 🔵 Pi Agent C (idle - 1h)
- ──────────────────
- Pin: Pi Agent A
- Pin: Pi Agent B
- ──────────────────
- Quit Splatter
```

### 8. Notification Engine

Cross-platform notification dispatch with configurable triggers.

```rust
struct NotificationEngine {
    config: NotificationConfig,
    active_notifications: HashMap<String, NotificationHandle>,
}

struct NotificationConfig {
    enabled: bool,
    sound: bool,                    // Play sound with notification
    focus_requirement: FocusPolicy, // Only when app not focused
    coalesce: CoalescePolicy,       // Group notifications within time window
    triggers: Vec<NotificationTrigger>,
}

enum FocusPolicy {
    Never,                    // Always
    WhenNotFocused,           // Only when Splatter not focused
    WhenMinimized,            // Only when Splatter minimized
    WhenInactiveMinutes(u32), // Only after N minutes of inactivity
}

enum CoalescePolicy {
    Off,                         // Each event is a separate notification
    Window(Duration),            // Group events within time window
    ByEvent(String),             // Group by event type (e.g., all "status_change" events)
    MaxCount(u32),               // Maximum notifications per window
}
```

**Configurable triggers:**

```rust
struct NotificationTrigger {
    id: String,
    name: String,
    description: String,
    enabled: bool,
    event: AgentEvent,      // What event triggers this
    condition: Option<String>, // Optional condition (e.g., status == "blocked")
    sound: bool,            // Play sound
    push: bool,             // Native OS notification
    tray: bool,             // Update tray
}

// Default triggers (all enabled):
triggers: [
    { id: "agent_blocked", name: "Agent blocked", event: "agent_status", condition: "status==blocked", sound: true, push: true, tray: true },
    { id: "agent_done", name: "Agent done", event: "agent_status", condition: "status==done", sound: false, push: true, tray: false },
    { id: "agent_working", name: "Agent working", event: "agent_status", condition: "status==working", sound: false, push: true, tray: false },
    { id: "agent_crash", name: "Agent crashed", event: "agent_exit", condition: "exit_code!=0", sound: true, push: true, tray: true },
    { id: "agent_long_running", name: "Long-running agent", event: "agent_duration", condition: "duration>30m", sound: false, push: true, tray: false },
    { id: "agent_connection_lost", name: "Connection lost", event: "connection_lost", condition: "true", sound: true, push: true, tray: true },
    { id: "agent_connection_restored", name: "Connection restored", event: "connection_restored", condition: "true", sound: false, push: true, tray: false },
]
```

**Platform-specific notification dispatch:**

| Platform | Method | Crate |
|----------|--------|-------|
| Linux (GNOME/KDE) | D-Bus `org.freedesktop.Notifications` | `zbus` |
| Linux (generic) | `notify-send` via `tauri-plugin-shell` | `tauri-plugin-shell` |
| macOS | `UserNotifications` framework | `usernotifications` |
| macOS (fallback) | `osascript` | `tauri-plugin-shell` |

### 9. Global Hotkey Registry

System-wide keyboard shortcuts via Tauri's `global-shortcut` plugin.

```rust
struct HotkeyRegistry {
    app: AppHandle,
    hotkeys: HashMap<String, GlobalShortcut>,
}

impl HotkeyRegistry {
    fn register_all(&mut self) {
        // Navigation
        self.register("nav-prev-pane", "CmdOrOpt+Shift+Up");
        self.register("nav-next-pane", "CmdOrOpt+Shift+Down");
        self.register("nav-cycle-next", "CmdOrOpt+Tab");
        self.register("nav-cycle-prev", "CmdOrOpt+Shift+Tab");
        self.register("nav-focus-left", "CmdOrOpt+H");
        self.register("nav-focus-down", "CmdOrOpt+J");
        self.register("nav-focus-up", "CmdOrOpt+K");
        self.register("nav-focus-right", "CmdOrOpt+L");

        // Layout
        self.register("layout-split-down", "CmdOrOpt+Shift+V");
        self.register("layout-split-right", "CmdOrOpt+Shift+D");
        self.register("layout-zoom-toggle", "CmdOrOpt+Z");
        self.register("layout-new-tab", "CmdOrOpt+Shift+T");
        self.register("layout-close-pane", "CmdOrOpt+Shift+X");
        self.register("layout-new-pane", "CmdOrOpt+Shift+P");

        // Agent actions
        self.register("agent-interrupt", "CmdOrOpt+I");
        self.register("agent-continue", "CmdOrOpt+C");
        self.register("agent-resume", "CmdOrOpt+R");
        self.register("agent-pin", "CmdOrOpt+Shift+P");
        self.register("agent-next-pinned", "CmdOrOpt+Shift+Shift+P");

        // App-level
        self.register("app-show-hide", "CmdOrOpt+S");
        self.register("app-toggle-sidebar", "CmdOrOpt+B");
        self.register("app-new-window", "CmdOrOpt+N");
        self.register("app-new-window-focus", "CmdOrOpt+Shift+N");
    }
}
```

### 10. Plugin System

A sandboxed plugin host with manifest, marketplace, and API.

**Plugin architecture:**

```rust
// Plugin manifest (plugin.yaml)
name: "agent-notifier"
version: "1.0.0"
description: "Send agent status to Discord/Slack/Telegram"
author: "splatter-user"
homepage: "https://github.com/user/splatter-agent-notifier"
license: "MIT"
entry: "main.js"        // JavaScript entry point
permissions: ["notification", "agent:read", "http"]

// Plugin lifecycle hooks
// Plugin exposes these functions in JavaScript:
async onPluginReady() { /* Plugin loaded, app is ready */ }
async onAgentStatusChanged(agent: AgentState) { /* Handle agent status change */ }
async onAgentActivity(agentId: string, activity: ActivityEntry) { /* Handle agent activity */ }
async onPluginWillUnload() { /* Cleanup */ }
```

**Plugin API (Rust → JS bridge):**

```typescript
// Available in plugin JavaScript
import { splatter } from '@splatter/plugin';

// Agent operations
await splatter.agent.getStatus(agentId);
await splatter.agent.getInterrupt(agentId);
await splatter.agent.resume(agentId);
await splatter.agent.list();

// Window/pane operations
await splatter.window.getActive();
await splatter.window.create();
await splatter.pane.list();
await splatter.pane.focus(paneId);

// Settings
await splatter.settings.get('key');
await splatter.settings.set('key', value);

// Notification
await splatter.notification.send({ title, body, sound });

// HTTP
const response = await splatter.http.fetch(url, options);

// Events
splatter.events.on('agent-status', handler);
splatter.events.on('window-close', handler);
```

**Plugin marketplace:**

- GitHub-hosted registry (like npm for plugins)
- Each plugin is a GitHub repo with a `plugin.yaml` manifest
- `splatter plugin search <query>` — search marketplace
- `splatter plugin install <name>` — install from marketplace
- `splatter plugin update <name>` — update plugin
- `splatter plugin uninstall <name>` — remove plugin

### 11. Settings & Configuration

Structured config.toml + in-app Settings UI + import/export.

```toml
# ~/.config/splatter/config.toml

[app]
name = "Splatter"
version = "0.1.0"
theme = "dark"
auto_update = true
crash_reporting = true

[windows]
default_width = 1400
default_height = 900
default_monitors = "primary"
max_windows = 10
restore_on_start = true
window_positions = true

[terminal]
font_family = "JetBrains Mono"
font_size = 14
cursor_style = "block"
cursor_blink = true
background_opacity = 1.0
scrollback_lines = 10000
input_batch_delay_ms = 32
output_coalesce_ms = 16

[agents]
default_working_dir = "inherit"
auto_detect = true
status_refresh_interval_ms = 200
agent_profiles_dir = "~/.config/splatter/agents"
resume_on_crash = true

[notifications]
enabled = true
sound = true
focus_policy = "when-not-focused"
coalesce_window_ms = 30000
max_per_window = 5

[hotkeys]
# See hotkey registry section

[tray]
enabled = true
show_on_start = true
color_icon = true
show_agent_count = true
menu_on_left_click = false
```

**Settings migration:**

```rust
// Settings versions migrate automatically
struct SettingsMigration {
    from_version: u32,
    to_version: u32,
    migration: fn(Settings) -> Settings,
}
```

### 12. Auto-Update + Crash Reporting

**Auto-updater:**

```toml
# src-tauri/tauri.conf.json (partial)
{
  "updater": {
    "active": true,
    "endpoints": [
      "https://api.github.com/repos/splatter-app/splatter/releases/latest"
    ],
    "dialog": false,
    "pubkey": "<ed25519-pubkey>"
  }
}
```

**Crash reporting:**

- Rust: `sentry` crate for crash dumps
- JS: Sentry JS SDK for WebView crashes
- Configurable in Settings → Diagnostic → Enable crash reporting

### 13. CI/CD & Distribution

**GitHub Actions workflow:**

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  release:
    strategy:
      matrix:
        platform: [ubuntu-24.04, macos-14, windows-latest]
    runs-on: ${{ matrix.platform }}

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 22

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_KEY_PASSWORD }}

      - name: Build Linux packages
        if: matrix.platform == 'ubuntu-24.04'
        run: |
          # Build .deb and .AppImage
          cargo tauri build --target x86_64-unknown-linux-gnu

      - name: Create Homebrew formula
        if: matrix.platform == 'macos-14'
        run: |
          # Generate Homebrew cask formula
          scripts/gen-cask.sh

      - name: Create AUR PKGBUILD
        if: matrix.platform == 'ubuntu-24.04'
        run: |
          # Generate AUR PKGBUILD
          scripts/gen-aur.sh

      - name: Upload artifacts
        uses: softprops/action-gh-release@v2
        with:
          files: |
            src-tauri/target/release/bundle/**/*
```

**Distribution targets:**

- GitHub Releases (tarballs, .deb, AppImage)
- Homebrew (macOS)
- AUR (Linux Arch)
- winget (Windows, future)

## File Structure

```
splatter/                              # Project root
├── Cargo.toml                         # Rust workspace root
├── package.json                       # Node.js deps (for web/)
├── src/                               # Rust source
│   ├── main.rs                        # Entry point
│   ├── lib.rs                         # Library root
│   ├── app/                           # App initialization
│   │   ├── mod.rs
│   │   ├── config.rs                  # Config loading
│   │   └── state.rs                   # Global app state
│   ├── agent/                         # Agent management
│   │   ├── mod.rs
│   │   ├── launcher.rs                # Process spawning
│   │   ├── manager.rs                 # Agent lifecycle
│   │   ├── detector.rs                # Agent type detection
│   │   ├── profile.rs                 # Agent profiles (YAML)
│   │   ├── history.rs                 # Activity history
│   │   └── resume.rs                  # Agent resume
│   ├── terminal/                      # Terminal management
│   │   ├── mod.rs
│   │   ├── session.rs                 # PTY sessions
│   │   ├── input.rs                   # Input forwarding
│   │   ├── output.rs                  # Output forwarding
│   │   └── resize.rs                  # Resize handling
│   ├── layout/                        # Layout engine
│   │   ├── mod.rs
│   │   ├── bsp.rs                     # Binary space partition
│   │   ├── ops.rs                     # Layout operations
│   │   └── presets.rs                 # Preset loading/saving
│   ├── tray/                          # System tray
│   │   ├── mod.rs
│   │   ├── builder.rs                 # Tray setup
│   │   └── menu.rs                    # Tray menu items
│   ├── hotkeys/                       # Global hotkeys
│   │   ├── mod.rs
│   │   ├── registry.rs                # Registration
│   │   └── handlers.rs                # Hotkey callbacks
│   ├── notify/                        # Notification engine
│   │   ├── mod.rs
│   │   ├── engine.rs                  # Dispatch logic
│   │   ├── trigger.rs                 # Trigger evaluation
│   │   ├── config.rs                  # Config loading
│   │   └── platform/                  # Platform dispatch
│   │       ├── mod.rs
│   │       ├── linux.rs               # D-Bus
│   │       ├── linux-cli.rs           # notify-send
│   │       └── macos.rs               # UserNotifications
│   ├── window/                        # Window management
│   │   ├── mod.rs
│   │   ├── manager.rs                 # Multi-window
│   │   ├── state.rs                   # Window persistence
│   │   └── monitor.rs                 # Monitor detection
│   ├── plugin/                        # Plugin system
│   │   ├── mod.rs
│   │   ├── host.rs                    # Plugin host (JS)
│   │   ├── manifest.rs                # Manifest parsing
│   │   ├── api.rs                     # Plugin API bridge
│   │   └── marketplace.rs             # Marketplace client
│   ├── settings/                      # Settings
│   │   ├── mod.rs
│   │   ├── store.rs                   # Settings storage
│   │   ├── migration.rs               # Version migrations
│   │   ├── schema.rs                  # Schema validation
│   │   └── import.rs                  # Import/export
│   ├── updater/                       # Auto-updater
│   │   ├── mod.rs
│   │   ├── client.rs                  # Update checks
│   │   └── installer.rs               # Install updates
│   └── crash/                         # Crash reporting
│       ├── mod.rs
│       ├── logger.rs                  # Error logging
│       └── sentry.rs                  # Sentry integration
├── src-tauri/                         # Tauri config
│   ├── tauri.conf.json               # App config
│   ├── tauri.linux.conf.json         # Linux overrides
│   ├── capabilities/                  # WebView permissions
│   │   └── default.json
│   ├── icons/                         # App icons
│   │   ├── 32x32.png
│   │   ├── 128x128.png
│   │   ├── 128x128@2x.png
│   │   ├── icon.icns
│   │   └── icon.ico
│   ├── build.rs                       # Build script
│   └── Cargo.toml                     # Tauri-specific deps
├── web/                               # Frontend (React + TypeScript)
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── eslint.config.js
│   ├── index.html
│   ├── public/                        # Static assets
│   └── src/                           # React source
│       ├── main.tsx                    # Entry point
│       ├── App.tsx                     # Root component
│       ├── components/                 # React components
│       │   ├── Sidebar/
│       │   │   ├── AgentList.tsx
│       │   │   ├── AgentGroup.tsx
│       │   │   ├── AgentPin.tsx
│       │   │   ├── AgentStatus.tsx
│       │   │   ├── SidebarNav.tsx
│       │   │   └── SidebarPanel.tsx
│       │   ├── Terminal/
│       │   │   ├── TerminalPane.tsx
│       │   │   ├── TerminalContainer.tsx
│       │   │   ├── TerminalToolbar.tsx
│       │   │   ├── TerminalResizeHandle.tsx
│       │   │   └── TerminalSelection.tsx
│       │   ├── Layout/
│       │   │   ├── LayoutGrid.tsx
│       │   │   ├── SplitHandle.tsx
│       │   │   ├── LayoutPresets.tsx
│       │   │   └── LayoutToolbar.tsx
│       │   ├── Status/
│       │   │   ├── StatusBar.tsx
│       │   │   ├── StatusBadge.tsx
│       │   │   └── StatusLegend.tsx
│       │   ├── Settings/
│       │   │   ├── SettingsPanel.tsx
│       │   │   ├── SettingsSections.tsx
│       │   │   ├── SettingsAgents.tsx
│       │   │   ├── SettingsNotifications.tsx
│       │   │   ├── SettingsHotkeys.tsx
│       │   │   ├── SettingsPlugins.tsx
│       │   │   ├── SettingsImportExport.tsx
│       │   │   └── SettingsCrash.tsx
│       │   ├── Plugin/
│       │   │   ├── PluginPanel.tsx
│       │   │   ├── PluginCard.tsx
│       │   │   ├── PluginSearch.tsx
│       │   │   └── PluginMarketplace.tsx
│       │   ├── Notification/
│       │   │   ├── NotificationToast.tsx
│       │   │   └── NotificationSettings.tsx
│       │   └── Window/
│       │       ├── WindowManager.tsx
│       │       └── WindowSelector.tsx
│       ├── hooks/                      # React hooks
│       │   ├── useAgent.ts             # Agent state
│       │   ├── useTerminal.ts          # Terminal state
│       │   ├── useLayout.ts            # Layout state
│       │   ├── useHotkeys.ts           # Hotkey handling
│       │   ├── useSettings.ts          # Settings
│       │   ├── useWindow.ts            # Window management
│       │   ├── usePlugin.ts            # Plugin management
│       │   └── useTray.ts              # System tray
│       ├── api/                        # Tauri API calls
│       │   ├── agent.ts                # Agent commands
│       │   ├── terminal.ts             # Terminal commands
│       │   ├── layout.ts               # Layout commands
│       │   ├── tray.ts                 # Tray commands
│       │   ├── hotkeys.ts              # Hotkey commands
│       │   ├── settings.ts             # Settings commands
│       │   ├── window.ts               # Window commands
│       │   └── plugin.ts               # Plugin commands
│       ├── events/                     # Event system
│       │   ├── emitter.ts              # Event emitter
│       │   ├── agent.ts                # Agent events
│       │   ├── terminal.ts             # Terminal events
│       │   └── window.ts               # Window events
│       ├── types/                      # TypeScript types
│       │   ├── agent.ts
│       │   ├── terminal.ts
│       │   ├── layout.ts
│       │   ├── settings.ts
│       │   └── plugin.ts
│       ├── themes/                     # Theme system
│       │   ├── dark.ts                 # Dark theme
│       │   └── index.ts
│       └── utils/                      # Utilities
│           ├── format.ts               # Time formatting
│           ├── debounce.ts             # Debounce helper
│           └── events.ts               # Event helpers
├── scripts/                            # Build/utility scripts
│   ├── dev.sh                          # Quick dev start
│   ├── build.sh                        # Build script
│   ├── gen-cask.sh                     # Homebrew cask generator
│   ├── gen-aur.sh                      # AUR PKGBUILD generator
│   └── test.sh                         # Test runner
├── tests/                              # Tests
│   ├── integration/                    # Integration tests
│   ├── unit/                           # Unit tests
│   │   ├── agent/                      # Agent tests
│   │   ├── terminal/                   # Terminal tests
│   │   ├── layout/                     # Layout tests
│   │   ├── notification/               # Notification tests
│   │   ├── hotkeys/                    # Hotkey tests
│   │   ├── settings/                   # Settings tests
│   │   └── plugin/                     # Plugin tests
│   └── e2e/                            # End-to-end tests
│       ├── agent-launch.e2e.ts         # Agent launch tests
│       ├── layout-split.e2e.ts         # Layout split tests
│       ├── tray.e2e.ts                 # System tray tests
│       └── hotkey.e2e.ts               # Global hotkey tests
├── plugins/                            # Example plugins (for development)
│   └── example-agent-notifier/
│       ├── plugin.yaml
│       └── main.js
├── .github/                            # GitHub config
│   └── workflows/
│       └── release.yml                 # CI/CD pipeline
├── .config/                            # Config for development
│   └── splatter/                       # Development config
│       ├── config.toml
│       └── agents/                     # Dev agent profiles
│           ├── pi.yaml
│           └── master-ai-stack.yaml
├── docs/                               # Documentation
│   ├── architecture.md                 # Architecture docs
│   ├── build.md                        # Build instructions
│   ├── packaging.md                    # Packaging docs
│   ├── plugins.md                      # Plugin development
│   └── settings.md                     # Settings reference
├── .gitignore
├── README.md                           # Project README
└── SKETCH.md                           # This file
```

## Key Differences from herdr-web-dash

| Aspect | herdr-web-dash | Splatter |
|--------|----------------|----------|
| **Runtime** | Browser (Chrome/Firefox) | Embedded WebView (native) |
| **Backend** | Separate Rust HTTP process | Tauri commands (same process) |
| **Transport** | HTTP + WebSocket over localhost | Direct IPC (Tauri events/commands) |
| **Terminal** | ghostty-web npm in browser | ghostty-web in WebView (same code) |
| **Agent model** | Observer (no launch) | Launcher (spawns agents) |
| **Multi-session** | Multi-bridge support | Multi-window per monitor |
| **Remote** | SSH thin client | Not in v1 (planned later) |
| **Notifications** | None | Full native OS notifications |
| **Hotkeys** | App-level only | Global system-wide |
| **Tray** | None | Full status panel |
| **Plugin system** | None | Full marketplace |
| **Auto-update** | Manual | Built-in |
| **Install** | Tarball | .deb/.AppImage (v1) |
| **Dependencies** | Node.js + Rust + herdr | Just the binary (WebView bundled) |

## Testing Strategy

See separate TEST_PLAN.md document.

## Migration Path / Build Phases

See BUILD_PLAN.md document.

## Risks and Mitigations

### Risk 1: Ghostty-web in Tauri WebView has issues on Linux

**Impact:** High. The entire terminal rendering stack depends on this.
**Mitigation:** Test early (Phase 0). If ghostty-web fails, fall back to:

- `@xterm/xterm` (pure JS VT, no canvas, no GPU)
- Custom ghostty-web build with patched dependencies
- Native VT rendering via `alacritty_terminal` (Rust, not Ghostty)
**Detection:** Build a minimal PoC in Phase 0. If it works, we're green. If not, pivot.

### Risk 2: Agent detection is imperfect

**Impact:** Medium. Some agent types may not be detected reliably.
**Mitigation:** Use multiple detection signals (process name, working dir, env vars, output patterns). Allow manual agent type override. Log undetected agents for future detection rules.

### Risk 3: Multi-window state management is complex

**Impact:** Medium. Multiple windows share state, need to handle disconnects, reconnects.
**Mitigation:** Implement window state persistence early. Use Tauri's window events for lifecycle. Test with 3+ windows on 3+ monitors.

### Risk 4: Plugin system security

**Impact:** High. Plugins run with app permissions, could access user data.
**Mitigation:** Sandboxed JS runtime. Permission model (each plugin declares needed permissions). Code review for marketplace plugins. Rate limiting for HTTP requests.

### Risk 5: Terminal input latency

**Impact:** Medium. WebView→Rust→herdr→PTY→VT→WebView round-trip adds latency.
**Mitigation:**

- Input batching (32ms delay for normal typing, immediate for paste)
- Use binary frames instead of JSON for input/output
- Profile with real-world workloads (large outputs, rapid input)
- Consider direct VT rendering for input-heavy workloads

### Risk 6: GTK/WebKitGTK version compatibility

**Impact:** Medium. Different distros ship different WebKitGTK versions.
**Mitigation:** Test on Ubuntu 22.04, 24.04, Fedora, Arch. Use `webkit2gtk` >= 4.0 from Tauri docs. Graceful fallback for older WebKitGTK (reduce feature set).

### Risk 7: AppImage sandboxing issues

**Impact:** Low-Medium. AppImage runs in sandbox, may have issues with notifications, tray, etc.
**Mitigation:** Bundle required dependencies. Test AppImage in multiple desktop environments. Consider AppImage with `--appimage-extract-and-run` for development.

## Success Criteria

### MVP (v0.1.0)

1. ✅ Opens a window on Linux (GTK/WebKitGTK)
2. ✅ Launches a Pi agent into a terminal pane
3. ✅ Shows terminal output (ghostty-web rendering works)
4. ✅ Tracks agent status (idle/working/blocked/done)
5. ✅ Can split/merge/close panes
6. ✅ Agent-aware sidebar with status dots
7. ✅ Settings persisted via config.toml

### v0.5.0

1. ✅ System tray with status indicator
2. ✅ Native OS notifications
3. ✅ Global hotkeys (navigation, agent actions)
4. ✅ Layout presets (built-in)
5. ✅ Agent resume (one-click)
6. ✅ Settings UI (full panel)
7. ✅ Agent pinning + groups

### v1.0.0

1. ✅ Multi-window (per monitor)
2. ✅ Full agent awareness (history, timeline, stats)
3. ✅ Agent notes and annotations
4. ✅ Agent handoff
5. ✅ Auto-updater
6. ✅ Crash reporting
7. ✅ Plugin system (basic)
8. ✅ .deb and AppImage distribution
9. ✅ GitHub Actions CI/CD
10. ✅ Homebrew formula (macOS)
11. ✅ AUR PKGBUILD (Linux)

## Future Features (Post-v1)

- Remote SSH attach (thin client)
- Multi-session management (connect to multiple herdr/daemon sessions)
- macOS + Windows builds
- Agent-to-agent handoff (cross-agent)
- Terminal image display (sixel/lima)
- Record and replay sessions
- Multi-cursor input
- Custom themes (light/dark/user-defined)
- Plugin marketplace
- Agent performance analytics dashboard
- Scripting API (Python/Rust hooks)
- Screen reader support (accessibility)
