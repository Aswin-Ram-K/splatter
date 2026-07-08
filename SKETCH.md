# Splatter — Standalone CMUX-Style Desktop App

The product name is **Splatter** (capitalized in UI, lowercase in paths and package names).
The project directory is `splatter/`. All references to "herdr-native" in this doc have been
renamed to "Splatter".

## Vision

A standalone, native-feeling desktop application that provides a Herdr terminal multiplexer
interface (like tmux/CMUX for agents) with an agent-aware notification panel, distributed as a
single desktop package for Linux (first), macOS, and Windows.

No browser. No separate bridge process. No `localhost:8787` to visit. One binary, one icon in
the dock/panel/taskbar, and the app lives in its own window.

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                       Splatter (Tauri App)                       │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                   WebView (Web UI Layer)                   │  │
│  │                                                            │  │
│  │  ┌──────────────────────────────────────────────────────┐  │  │
│  │  │           Reused herdr-web React UI                  │  │  │
│  │  │                                                      │  │  │
│  │  │  ┌─────────┐  ┌──────────────────────────────────┐  │  │  │
│  │  │  │ Sidebar │  │       Terminal Grid (CMUX)       │  │  │  │
│  │  │  │         │  │                                   │  │  │  │
│  │  │  │ 🤖 Agent│  │  ┌────────┐  ┌────────┐         │  │  │  │
│  │  │  │ Status │  │  │ Pane 1 │  │ Pane 2 │         │  │  │  │
│  │  │  │ Panel │  │  │ (bash) │  │ (Claude) │         │  │  │  │
│  │  │  │         │  │  └────────┘  └────────┘         │  │  │  │
│  │  │  │         │  │  ┌────────┐  ┌────────┐         │  │  │  │
│  │  │  └─────────┘  │  │ Pane 3 │  │ Notes  │         │  │  │  │
│  │  │               │  │  └────────┘  └────────┘         │  │  │  │
│  │  │               └──────────────────────────────────┘  │  │  │
│  │  │                                                      │  │  │
│  │  └──────────────────────────────────────────────────────┘  │  │
│  │                                                            │  │
│  │  ┌──────────────────────────────────────────────────────┐  │  │
│  │  │  Native Shell (status bar + system tray)              │  │  │
│  │  └──────────────────────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │               Rust Core (Tauri Commands)                   │  │
│  │                                                            │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌─────────────┐  │  │
│  │  │ Herdr IPC│ │ Terminal │ │Notif.    │ │ Hotkey/     │  │  │
│  │  │ Manager  │ │ Renderer │ │ Engine   │ │ Native API  │  │  │
│  │  │          │ │          │ │          │ │             │  │  │
│  │  │• Unix    │ │• libghost│ │• macOS   │ │• Global     │  │  │
│  │  │  sockets │ │ ty-vt    │ │  badges  │ │   hotkeys   │  │  │
│  │  │• Named   │ │• Layout  │ │• Linux   │ │• Native     │  │  │
│  │  │  pipes   │ │   mgmt   │ │  Growl   │ │   menus     │  │  │
│  │  │• Proto  │ │• ANSI    │ │• Win     │ │• System     │  │  │
│  │  │  bridge  │ │   render │ │  Toast   │ │   tray      │  │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └─────────────┘  │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

## Why Tauri?

Tauri gives us the best of both worlds:

1. **Reuse the existing React/TypeScript UI** — the herdr-web codebase (TerminalView, agent panel,
   split layout, notes, etc.) runs inside the embedded WebView. Minimal changes needed.
2. **Native Rust core** — all actual herdr socket communication, terminal rendering, and OS
   integration lives in Rust, exposed to the WebView via Tauri commands.
3. **Cross-platform** — Linux (GTK/WebKitGTK), macOS (AppKit/WKWebView), Windows (Win32/WebView2)
   with one codebase.
4. **Small binary** — ~5-10 MB vs Electron's ~150+ MB.
5. **Rust-first** — fits naturally with the existing herdr-compat vendor crate.

## Component Breakdown

### 1. Terminal Rendering (Native)

The current bridge sends raw terminal frames via WebSocket to the browser's Ghostty-based renderer.
For native, we replace the WebSocket bridge with direct Rust→WebUI terminal output.

**Approach: embed Ghostty's libghostty-vt via Rust**

The herdr-compat already vendors herdr's protocol which itself uses libghostty-vt. We extend the
vendor crate to expose the VT emulation + rendering layer:

```rust
// In vendor/herdr-compat/src/lib.rs:
pub mod terminal; // New: VT emulation + rendering

// Usage from Rust core:
use herdr_compat::terminal::{TerminalEngine, RenderTarget};

let engine = TerminalEngine::new(cols, rows);
engine.feed(&ansi_bytes);          // Feed raw VT escape sequences
let bitmap = engine.render();      // Get rendered frame (RGBA)
// Or: let text = engine.text_content();  // For selection/copy
```

**Fallback:** If libghostty-vt FFI isn't practical, use `alacritty_terminal` (Rust VT emulator)
or implement a simple VT parser with `crossterm`/`termwiz` for basic rendering, then layer on
true color / mouse / bracketed paste later.

**Layout management:** The CMUX split layout (like tmux panes) is handled by the existing layout
engine in the bridge. We expose it as a Rust struct that computes pane rectangles, and the WebView
renders them as positioned terminal containers.

### 2. Herdr IPC Layer (Rust)

Replace the HTTP+WebSocket bridge with direct socket communication:

```rust
pub struct HerdrSession {
    socket_path: PathBuf,
    stream: LocalStream,           // From interprocess crate (already vendored)
    terminal_sessions: HashMap<String, TerminalSession>,
    agent_activity: AgentActivityStore,
    pane_selection: RwLock<Option<String>>,
}

impl HerdrSession {
    // Connect to herdr daemon
    pub fn connect(socket_path: &Path) -> Result<Self>;
    
    // Spawn terminal session (attach to pane)
    pub fn attach_terminal(&mut self, terminal_id: &str, cols: u16, rows: u16) -> Result<TerminalSession>;
    
    // Send input to active terminal
    pub fn send_input(&self, terminal_id: &str, data: &[u8]) -> Result<()>;
    
    // Resize terminal
    pub fn resize_terminal(&self, terminal_id: &str, cols: u16, rows: u16) -> Result<()>;
    
    // Subscribe to structural events (workspaces/tabs/panes)
    pub fn subscribe_events(&self, subscriptions: &[Subscription]) -> Result<EventStream>;
    
    // Subscribe to agent activity
    pub fn subscribe_activity(&self, subscriptions: &[Subscription]) -> Result<ActivityStream>;
    
    // Issue commands (split, move, close, rename, etc.)
    pub fn command(&self, method: Method, params: serde_json::Value) -> Result<serde_json::Value>;
    
    // Get current snapshot
    pub fn snapshot(&self) -> Result<Snapshot>;
}
```

The existing `vendor/herdr-compat/src/ipc.rs`, `api/`, and `protocol/` modules are already written
and tested. We reuse them directly — the bridge essentially does this same thing over HTTP/WSS.

### 3. Agent Notification Panel

The agent panel shows per-pane agent status with visual indicators:

```
┌────────────────────────────────────┐
│ Agent Status                       │
├────────────────────────────────────┤
│ 🟢 Claude   working   (2m ago)    │
│ 🟡 Codex    blocked   (5m ago)    │
│ 🔵 Pi       idle      (1h ago)    │
│ ⚪ bash     idle      (done)      │
│ 🟢 Codex    working   (just now)  │
│ ...                                │
└────────────────────────────────────┘
```

**Native notification features:**

- **macOS:** Native push notifications via `objc`/`macos` crates, or `launch_notification` from
  herdr-compat's sound module for sound alerts. Use AppKit's `NSUserNotification` (deprecated but
  still works) or `UserNotifications` framework.
- **Linux:** D-Bus desktop notifications via `zbus` or `dbus-common`.
- **Windows:** WinRT `Windows.UI.Notifications` toast notifications.
- **Agent badges:** macOS dock badge using AppKit to show unread agent count.
- **Focus notification:** When an agent goes `blocked` or `working`, flash the app icon or
  generate a system sound (reusing herdr-compat's sound module).

**Agent pinning:** Users can pin important agents to always-show in the sidebar with a persistent
notification strip at the top.

### 4. Window / Layout System

A single main window that can be resized and manages the CMUX terminal grid.

```rust
use tauri::Manager;

#[tauri::command]
fn set_split_layout(
    window: tauri::Window,
    layout: SplitLayout,
) {
    // Compute pane rectangles
    // Tell WebView to re-render with new layout
    window.emit("layout-changed", layout);
}

#[tauri::command]
fn set_window_title(window: tauri::Window, title: String) {
    window.set_title(&title).unwrap();
}

#[tauri::command]
fn zoom_terminal(window: tauri::Window, zoomed_pane: Option<String>) {
    // Toggle maximized view for a single pane
    window.emit("zoom-pane", ZoomedPane { id: zoomed_pane });
}
```

**Window behavior:**

- Single main window with a menu bar (not titlebar-based navigation)
- Resizable terminal grid with drag-resizable splitters (handled by WebView CSS/JS)
- Keyboard shortcuts for pane navigation (global hotkeys via Rust)
- "Always on top" toggle for focused agent pane
- Native context menus for pane actions

### 5. Global Hotkeys

```rust
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

fn register_hotkeys(app: tauri::AppHandle) {
    app.global_shortcut().register_all(&[
        GlobalShortcutConfig {
            shortcut: GlobalShortcut::new(ctl("Alt"), Key::ArrowUp),
            handler: move |_| { /* select previous pane */ },
        },
        GlobalShortcutConfig {
            shortcut: GlobalShortcut::new(ctl("Alt"), Key::ArrowDown),
            handler: move |_| { /* select next pane */ },
        },
        GlobalShortcutConfig {
            shortcut: GlobalShortcut::new(ctl("Alt"), Key::Tab),
            handler: move |_| { /* cycle split next */ },
        },
        GlobalShortcutConfig {
            shortcut: GlobalShortcut::new(ctl(ctl("Alt")), Key::KeyV),
            handler: move |_| { /* split down */ },
        },
        GlobalShortcutConfig {
            shortcut: GlobalShortcut::new(ctl(ctl("Alt")), Key::KeyH),
            handler: move |_| { /* focus left */ },
        },
        // ... H/J/K/L for directions
    ]);
}
```

### 6. Menu Bar

Native app-level menu bar (not browser menu):

```
┌────────────────────────────────────────────────────┐
│ Splatter  File    View      Window    Help     │
└────────────────────────────────────────────────────┘
```

| Menu | Items |
|------|-------|
| **File** | New Pane, New Tab, New Workspace, Split Right, Split Down, Close Pane, Close Tab, Disconnect |
| **View** | Toggle Sidebar, Toggle Notes, Zoom In, Zoom Out, Reset Zoom, Toggle Fullscreen |
| **Window** | Previous Pane, Next Pane, Focus Left/Right/Up/Down, Toggle Zoom, Minimize, Bring All to Front |
| **Help** | Documentation, Troubleshoot, About |

### 7. System Tray (Optional)

```rust
use tauri::menu::{Menu, MenuBuilder, MenuItem, MenuBuilderExt};

fn build_tray(app: &tauri::App) -> Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Splatter", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;
    
    let tray = trayicon::Builder::new()
        .with_menu(&menu)
        .with_tooltip("Splatter")
        .with_icon(app.default_window_icon()?.clone())
        .build(app)?;
    
    tray.on_tray_icon_event(|tray, event| {
        match event {
            trayicon::TrayIconEvent::Click { .. } => {
                tray.app().get_webview_window("main").unwrap().show().unwrap();
            }
            _ => {}
        }
    });
    
    Ok(())
}
```

## File Structure

```
Splatter/                          # New project directory
├── Cargo.toml                         # Tauri + Rust deps
├── src/                               # Rust code
│   ├── main.rs                        # Entry point
│   ├── app.rs                         # App configuration
│   ├── herdr/                         # Herdr IPC layer
│   │   ├── mod.rs
│   │   ├── session.rs                 # Main connection manager
│   │   ├── terminal.rs                # Per-terminal attach/detach
│   │   ├── events.rs                  # Event subscription/streaming
│   │   ├── commands.rs                # Workspace/tab/pane commands
│   │   └── snapshot.rs                # Snapshot aggregation
│   ├── terminal/                      # Terminal engine
│   │   ├── mod.rs
│   │   ├── engine.rs                  # VT emulation (libghostty-vt wrapper)
│   │   ├── layout.rs                  # Split layout calculations
│   │   └── renderer.rs                # Frame rendering (RGBA/bytes)
│   ├── notification/                  # Agent notifications
│   │   ├── mod.rs
│   │   ├── engine.rs                  # Cross-platform notification dispatch
│   │   ├── macos.rs
│   │   ├── linux.rs
│   │   └── windows.rs
│   ├── hotkeys/                       # Global hotkey management
│   │   ├── mod.rs
│   │   └── registry.rs
│   └── menu/                          # Native menu bar
│       ├── mod.rs
│       └── builder.rs
├── src-tauri/                         # Tauri config
│   ├── tauri.conf.json               # App config (name, version, icon, etc.)
│   ├── tauri.linux.conf.json         # Linux-specific overrides
│   ├── tauri.macos.conf.json         # macOS-specific overrides
│   ├── tauri.windows.conf.json       # Windows-specific overrides
│   ├── capabilities/                 # WebView capabilities
│   ├── icons/                        # App icons (all sizes)
│   ├── build.rs                      # Build script
│   └── Cargo.toml                    # Tauri-specific deps
├── web/                               # REUSED from herdr-web (symlink or copy)
│   ├── src/                           # Existing React UI code
│   ├── package.json
│   └── ...
├── vendor/                            # REUSED from herdr-web
│   └── herdr-compat/                  # Vendored Herdr compatibility
├── scripts/
│   ├── dev.sh                         # Quick dev script
│   └── build.sh                       # Build script
├── docs/
│   ├── architecture.md
│   ├── build.md
│   └── packaging.md
└── README.md
```

## Tauri Configuration

### `src-tauri/tauri.conf.json`

```json
{
  "productName": "Splatter",
  "version": "0.1.0",
  "identifier": "com.herdr.native",
  "build": {
    "frontendDist": "../web/dist",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "npm run dev --prefix ../web",
    "beforeBuildCommand": "npm run build --prefix ../web"
  },
  "app": {
    "windows": [
      {
        "title": "Splatter",
        "width": 1400,
        "height": 900,
        "resizable": true,
        "fullscreen": false,
        "decorations": true,
        "transparent": false,
        "focus": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self'; img-src 'self' data:; connect-src 'self' ws://localhost ipc://localhost"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["deb", "rpm", "appimage", "dmg", "nsis"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

### `src-tauri/Cargo.toml`

```toml
[package]
name = "Splatter"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = ["shell-open"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "sync", "time"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Tauri plugins
tauri-plugin-shell = "2"
tauri-plugin-global-shortcut = "2"
tauri-plugin-store = "2"
tauri-plugin-notification = "2"

# Platform-specific
[target.'cfg(target_os = "macos")'.dependencies]
objc = "0.2"
macos = "0.1"
usernotifications = "0.1"

[target.'cfg(target_os = "linux")'.dependencies]
zbus = "4"

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = ["Win32_UI_WindowsAndMessaging", "Win32_UI_Notifications", "Win32_Foundation"] }

# Herdr compat (vendored)
herdr-compat = { path = "../../vendor/herdr-compat" }

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

## WebView Integration (Rust ↔ React Bridge)

The WebView communicates with Rust via Tauri's event system and commands:

### Rust → WebView (events)

```rust
// When a terminal produces output
window.emit("terminal-output", TerminalOutputEvent {
    terminal_id: "term-1".to_string(),
    data: ansi_bytes,  // Vec<u8>
});

// When agent status changes
window.emit("agent-status", AgentStatusEvent {
    pane_id: "pane-42".to_string(),
    status: "working".to_string(),
    agent: "Claude".to_string(),
});

// When layout changes (split/resize)
window.emit("layout-update", LayoutUpdateEvent {
    splits: vec![...],
    panes: vec![...],
});
```

### WebView → Rust (commands)

```typescript
// In TypeScript (React side):
import { invoke } from '@tauri-apps/api/core';

// Send terminal input
await invoke('send_terminal_input', {
    terminalId: 'term-1',
    data: 'ls -la\n',  // string or base64 binary
});

// Resize terminal
await invoke('resize_terminal', {
    terminalId: 'term-1',
    cols: 120,
    rows: 40,
});

// Issue herdr commands
await invoke('herdr_command', {
    method: 'pane.split',
    params: { direction: 'down', target_pane_id: 'pane-42' },
});

// Get current snapshot
const snapshot = await invoke<Snapshot>('get_snapshot');
```

### Tauri Commands (Rust side)

```rust
use tauri::command;

#[command]
fn send_terminal_input(
    app: tauri::AppHandle,
    terminal_id: String,
    data: String,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    state.session.send_input(&terminal_id, data.as_bytes())
        .map_err(|e| e.to_string())
}

#[command]
fn resize_terminal(
    app: tauri::AppHandle,
    terminal_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    state.session.resize_terminal(&terminal_id, cols, rows)
        .map_err(|e| e.to_string())
}

#[command]
fn herdr_command(
    app: tauri::AppHandle,
    method: String,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let state = app.state::<AppState>();
    let result = state.session.command(&method, params)
        .map_err(|e| e.to_string())?;
    Ok(result)
}

#[command]
fn get_snapshot(app: tauri::AppHandle) -> Result<Snapshot, String> {
    let state = app.state::<AppState>();
    state.session.snapshot()
        .map_err(|e| e.to_string())
}

#[command]
fn subscribe_events(
    app: tauri::AppHandle,
    window: tauri::Window,
    subscriptions: Vec<String>,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let stream = state.session.subscribe_events(&subscriptions)
        .map_err(|e| e.to_string())?;
    
    tokio::spawn(async move {
        loop {
            match stream.next().await {
                Some(event) => {
                    let _ = window.emit("herdr-event", event);
                }
                None => break,
            }
        }
    });
    
    Ok(())
}
```

## Terminal Emulation: libghostty-vt

The herdr-compat vendor crate already depends on Ghostty/VT for encoding. We extract the VT
emulation layer:

```rust
// vendor/herdr-compat/src/terminal/engine.rs

pub struct TerminalEngine {
    // VT state machine instance
    inner: VTEngine,
    // Rendered output (text grid + styling)
    cells: Vec<Cell>,
    // Dimensions
    width: u16,
    height: u16,
}

impl TerminalEngine {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            inner: VTEngine::new(),
            cells: vec![Cell::default(); (width * height) as usize],
            width,
            height,
        }
    }
    
    // Feed raw VT escape sequences (from herdr)
    pub fn feed(&mut self, data: &[u8]) {
        self.inner.input(data);
    }
    
    // Get current rendered frame
    pub fn render(&self) -> RenderFrame {
        RenderFrame {
            width: self.width,
            height: self.height,
            cells: self.cells.clone(),
        }
    }
    
    // For text selection and copy/paste
    pub fn text_content(&self, selection: Option<Selection>) -> String {
        // ...
    }
}
```

**Rendering approach in WebView:**

The WebView renders terminal panes using canvas elements. Each frame:

1. Rust emits terminal output events with VT escape sequences
2. WebView's terminal renderer (Ghostty Web / `ghostty-web` npm package) renders to canvas
3. Or: Rust renders to bitmap, WebView displays as `<img>` or canvas

**Two options:**

| Option | Pros | Cons |
|--------|------|------|
| **A. WebView renders VT** | Reuse existing `ghostty-web` npm package; full feature parity | Terminal rendering not fully native; still depends on JS renderer |
| **B. Rust renders bitmap** | Fully native rendering pipeline; WebView just displays frames | More work; bitmap transfer overhead; need to re-implement VT rendering in Rust |

**Recommendation: Start with Option A** (reuse ghostty-web). It's the fastest path to a working
app. The terminal rendering quality is already excellent. We can later layer on native rendering
for performance-sensitive cases (large outputs, high-DPI, GPU acceleration).

## Platform-Specific Considerations

### Linux

- **Window:** GTK+4 via WebKitGTK (Tauri's default on Linux)
- **Notifications:** D-Bus via `zbus` crate → `org.freedesktop.Notifications`
- **Icon:** Desktop entry file in `.local/share/applications`
- **Package formats:** `.deb`, `.rpm`, `.AppImage`
- **IME:** WebKitGTK handles IME for CJK input
- **Wayland vs X11:** Tauri/WebkitGTK handles both; test on both

### macOS

- **Window:** AppKit + WKWebView (Tauri's default on macOS)
- **Notifications:** `UserNotifications` framework via `usernotifications` crate
- **Dock badge:** AppKit `NSApplication` badge number
- **Agent status in title bar:** Custom titlebar with agent status indicator
- **Package formats:** `.dmg`, `.pkg`
- **Gatekeeper:** Notarization for distribution
- **Universal binary:** `x86_64` + `arm64`

### Windows

- **Window:** Win32 + WebView2 (Edge/Chromium)
- **Notifications:** WinRT `Windows.UI.Notifications` API
- **Package formats:** `.msi`, `.exe` (NSIS installer)
- **Unicode:** UTF-8 everywhere; handle CJK via WebView2
- **Antivirus:** Windows Defender may flag unsigned binaries

## Migration Path from herdr-web-dash

### Phase 0: Vendor Setup (1-2 days)

- Copy `vendor/herdr-compat` from herdr-web-dash
- Copy `web/` from herdr-web-dash (or symlink)
- Setup Tauri project scaffolding
- Verify: `tauri dev` shows the existing herdr-web UI

### Phase 1: Bridge Replacement (1-2 weeks)

- Replace HTTP bridge calls with Tauri commands
- Herdr socket communication moves from bridge.rs to Rust core
- WebSocket terminal sessions → Rust→WebView direct event streaming
- Event subscriptions → Tauri events

### Phase 2: Native Features (2-3 weeks)

- Global hotkeys (Tauri global-shortcut plugin)
- System tray (custom tray implementation)
- Native notifications (platform-specific crates)
- Agent status badges on macOS dock / Linux taskbar / Windows tray
- Native menu bar (Tauri menu plugin)

### Phase 3: Polish (2-4 weeks)

- Terminal rendering optimization
- Window management (zoom, always-on-top)
- Settings/preferences UI (persisted via Tauri store plugin)
- Crash reporting
- Update mechanism (Tauri updater plugin)

### Phase 4: Distribution (ongoing)

- Package signing
- Auto-updater configuration
- GitHub Actions CI/CD for Linux/macOS/Windows releases
- Homebrew cask (macOS)
- AUR package (Linux)

## Key Differences from Current herdr-web-dash

| Aspect | herdr-web-dash (current) | Splatter (proposed) |
|--------|-------------------------|------------------------|
| **Runtime** | Browser (Chrome/Firefox) | Embedded WebView (native) |
| **Bridge** | Separate Rust HTTP process | Tauri commands (same process) |
| **Transport** | HTTP + WebSocket over localhost | Direct IPC (Tauri events/commands) |
| **Terminal** | ghostty-web npm in browser | ghostty-web in WebView (same code) |
| **Notifications** | None (web-only) | Native OS notifications |
| **Hotkeys** | App-level keyboard shortcuts | Global system-wide hotkeys |
| **Icon** | Browser tab icon | Dock/panel/taskbar icon |
| **Tray** | None | System tray with agent status |
| **Menu** | Browser menu | Native app menu bar |
| **Install** | Tarball + manual start | .deb/.dmg/.msi installer |
| **Binary size** | ~10 MB (Rust) + Node.js | ~5-15 MB (Tauri) |
| **Dependencies** | Node.js + Rust | Just the binary (WebView bundled) |

## Risks and Mitigations

### Risk 1: libghostty-vt is not easily FFI-exposed

**Mitigation:** Start with ghostty-web in WebView (no native VT rendering needed). The bridge
already works with JSON/WebSocket transport. The native app just changes the transport layer.

### Risk 2: Tauri WebView limitations on Linux

**Mitigation:** WebKitGTK on Linux is mature. Test terminal rendering carefully. If needed, fall
back to X11 mode or use a different WebView backend.

### Risk 3: macOS notarization overhead

**Mitigation:** Get Apple Developer program membership early. Automate notarization in CI/CD.

### Risk 4: Herdr protocol changes break compatibility

**Mitigation:** The herdr-compat vendor crate already handles this. The same vendor refresh
process applies. Protocol version test in herdr-compat catches regressions.

### Risk 5: Terminal input latency

**Mitigation:** WebView→Rust→herdr→VT→WebView round-trip adds latency. Optimize by:

- Batching input events
- Using binary frames instead of JSON
- Profiling with real-world workloads

## Success Criteria

1. **Works:** Opens a window, connects to herdr, displays terminal panes
2. **Splits:** Can create/h navigate/resize split panes
3. **Agent-aware:** Shows agent status panel with real-time updates
4. **Notifications:** Generates native notifications for agent events
5. **Hotkeys:** Global keyboard shortcuts work for pane navigation
6. **Cross-platform:** Runs on Linux (GTK), macOS (AppKit), Windows (Win32)
7. **Installable:** Distributes as proper packages (.deb/.dmg/.msi)
8. **Standalone:** No browser required, no separate bridge process

## Naming

Current project: **herdr-web** / **herdr-web-dash** / **herdr-web-dash-bridge**

Proposed: **Splatter** (or **herdr-desktop**)

The name should communicate "native desktop app, not a web app" while staying in the herdr
ecosystem.
