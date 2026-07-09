# Splatter — Build Plan

## Product

A standalone, agent-aware terminal multiplexer built on **Tauri 2 (Webview)**, **React + TypeScript**, and **Ghostty-web** (WASM). Rebuilt from scratch — no dependency on the herdr daemon or herdr-web-dash application.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Tauri Shell                         │
│  (Rust backend — windows, tray, hotkeys, config, IPC)   │
├─────────────────────────────────────────────────────────┤
│                     Tauri IPC                           │
├─────────────────────────────────────────────────────────┤
│                     Webview                             │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────┐ │
│  │  React 19    │  │   Zustand    │  │   Ghostty-web  │ │
│  │  (UI Layer)  │  │  (State Mgr) │  │ (Terminal)     │ │
│  ├─────────────┤  ├──────────────┤  ├────────────────┤ │
│  │ Agent List  │  │ Layout Store │  │ Pane Renderer  │ │
│  │ Agent List  │  │ Agent Store  │  │ PTY Bridge     │ │
│  │ Status Bar  │  │ Settings     │  │ VT Parser      │ │
│  │ Settings    │  │ Config Store │  │ Canvas Render  │ │
│  │ Tray Popup  │  │ Tray Store   │  │ Input Handler  │ │
│  │ Hotkey List │  │ Tray Store   │  │ Link Detector  │ │
│  │ Plugins     │  │ Plugin Store │  │ Selection      │ │
│  │ Crash Logs  │  │ Crash Store  │  │ Buffer NS      │ │
│  └─────────────┘  └──────────────┘  └────────────────┘ │
├─────────────────────────────────────────────────────────┤
│                    Tauri Backend                        │
│  ┌──────────┐  ┌─────────┐  ┌──────────┐  ┌────────┐  │
│  │ Agent Spawner │ │ Layout Engine │  │ Tray Manager │  │ Hotkey │  │
│  │ PTY (rustix)  │  │ BSP Tree     │  │ Status Color │  │ Global │  │
│  ├──────────┤  ├─────────┤  ├──────────┤  ├────────┤  │
│  │ Config Loader │ │ Window Mgr  │  │ Notification│  │ Plugin │  │
│  │ TOML + Schema │  │ Monitor Db  │  │ D-Bus / cli│  │ Host   │  │
│  ├──────────┤  ├─────────┤  ├──────────┤  ├────────┤  │
│  │ Auto Updater │ │ Crash Rep │  │ Plugin Host │  │ Settings│  │
│  │ Tauri updater │  │ Sentry   │  │ Manifest │   │ UI     │  │
│  └──────────┘  └─────────┘  └──────────┘  └────────┘  │
├─────────────────────────────────────────────────────────┤
│                Platform Integration                     │
│  ┌─────────┐  ┌─────────┐  ┌──────────┐  ┌────────┐  │
│  │ Linux   │  │ macOS   │  │ Windows  │  │ Remote │  │
│  │ GTK/    │  │ AppKit  │  │ WebView2 │  │ SSH    │  │
│  │ WebKit  │  │ Tray    │  │ Tray     │  │ (v2)   │  │
│  │ D-Bus   │  │ D-Bus   │  │ Registry │  │ (Future│  │
│  │ notify  │  │ notify  │  │ settings │  │ only)  │  │
│  └─────────┘  └─────────┘  └──────────┘  └────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Technology Choices

### Core Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Runtime | **Tauri 2** (Rust backend + Webview) | Native window management, system tray, global hotkeys, small binary |
| Web Layer | **React 19** + TypeScript | Mature UI ecosystem, hooks, concurrent features |
| State | **Zustand** + Immer | Minimal boilerplate, devtools, middleware (persist, subscribeWithSelector) |
| Terminal | **Ghostty-web** (WASM) | Proven at scale (Coder/Tabpad), canvas rendering, VT parsing |
| Config | **TOML** (config.toml) | Simple, structured, human-editable, version-control friendly |
| Agent Profiles | **YAML** (profiles/*.yaml) | Readable, structured, supports complex configurations |

### Key Dependencies

#### Rust (Backend)

| Crate | Version | Purpose |
|-------|---------|---------|
| `tauri` | 2.x | Core Tauri bindings |
| `tauri-plugin-window` | 2.x | Window management |
| `tauri-plugin-global-shortcut` | 2.x | Global hotkeys |
| `tauri-plugin-tray-icon` | 2.x | System tray |
| `tauri-plugin-updater` | 2.x | Auto-updater |
| `tauri-plugin-shell` | 2.x | PTY spawning |
| `rustix` | 0.38+ | Linux PTY (libpty, termios) |
| `serde` | 1.x | Serialization (TOML/JSON) |
| `serde_toml` | 0.8+ | TOML config parsing |
| `notify` | 8.x | Config file watcher |
| `notify-debouncer-full` | 0.5+ | Debounced file watching |
| `dirs` | 5.x | Platform-specific paths |
| `log` + `env_logger` | 0.11+ | Logging |
| `anyhow` + `thiserror` | 1.x | Error handling |
| `uuid` | 1.x | Agent IDs |
| `chrono` | 0.4+ | Timestamps |
| `futures` | 0.3+ | Async PTY read |
| `crossbeam-channel` | 0.5+ | Sync channels |
| `parking_lot` | 0.12+ | Thread-safe locks |
| `zbus` | 4.x | D-Bus notifications (Linux only) |
| `sentry` | 0.34+ | Crash reporting |

#### JavaScript/TypeScript (Frontend)

| Package | Version | Purpose |
|---------|---------|---------|
| `react` | 19.x | UI framework |
| `react-dom` | 19.x | DOM rendering |
| `typescript` | 5.x | Type safety |
| `zustand` | 5.x | State management |
| `immer` | 10.x | Immutable state updates |
| `ghostty-web` | latest | Terminal emulator (WASM) |
| `@tauri-apps/api` | 2.x | Tauri JS bindings |
| `class-variance-authority` | latest | CSS class management |
| `tailwindcss` | 4.x | Utility-first CSS |
| `@headlessui/react` | 2.x | Unstyled UI components |
| `@radix-ui/*` | latest | Headless primitives |
| `date-fns` | latest | Date formatting |
| `file-icons` | latest | File-type icons for agent list |

#### Tooling

| Package | Purpose |
|---------|---------|
| `vite` | Build tool |
| `vitest` | Unit testing |
| `playwright` | E2E testing |
| `tauri-cli` | Build/release |

---

## Build Phases

### Phase 0: Scaffolding (v1–v2 days)

Set up the project structure, dependencies, and base configuration.

#### Step 0.1: Tauri 2 Project Init

```bash
# Create Tauri 2 + React project
cargo init --name Splatter --lib splatter-core
cd splatter-core
```

Cargo.toml dependencies:

```toml
[package]
name = "splatter-core"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
tauri-plugin-global-shortcut = "2"
tauri-plugin-tray-icon = "2"
tauri-plugin-updater = "2"
rustix = { version = "0.38", features = ["process", "pty", "termios"] }
serde = { version = "1", features = ["derive"] }
serde_toml = "0.8"
dirs = "5"
log = "0.4"
env_logger = "0.11"
anyhow = "1"
thiserror = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
futures = "0.3"
crossbeam-channel = "0.5"
parking_lot = "0.12"
```

#### Step 0.2: Web Layer Setup

```bash
# Vite + React + TypeScript project
npm create vite@latest web -- --template react-ts
cd web
npm install
```

#### Step 0.3: Tauri Integration

Configure `src-tauri/tauri.conf.json`:

```json
{
  "productName": "Splatter",
  "version": "0.1.0",
  "identifier": "com.splatter.app",
  "build": {
    "frontendDist": "../web/dist",
    "devUrl": "http://localhost:1420"
  },
  "app": {
    "windows": [
      {
        "title": "Splatter",
        "width": 1280,
        "height": 720,
        "resizable": true,
        "transparent": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; img-src 'self' asset: https://asset.localhost; style-src 'self' 'unsafe-inline';"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["deb", "appimage"],
    "icon": []
  }
}
```

#### Step 0.4: Ghostty-web Integration

Install and configure ghostty-web in web/ directory:

```bash
cd web
npm install ghostty-web
```

Create initial terminal component to verify WASM loads:

```typescript
// web/src/components/GhosttyTerminal.tsx
import { useEffect, useRef } from 'react';
import { init, Terminal } from 'ghostty-web';

export function GhosttyTerminal({ cols = 80, rows = 24 }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);

  useEffect(() => {
    let term: Terminal;

    (async () => {
      await init({}); // Load WASM
      term = new Terminal({ cols, rows });
      term.open(containerRef.current!);
      term.onData(data => console.log('Input:', data));
      termRef.current = term;
    })();

    return () => {
      term?.dispose();
    };
  }, [cols, rows]);

  return <div ref={containerRef} style={{ width: '100%', height: '100%' }} />;
}
```

#### Step 0.5: Initial Web Structure

```
web/
├── src/
│   ├── components/
│   │   ├── GhosttyTerminal.tsx
│   │   ├── Layout.tsx
│   │   ├── StatusBar.tsx
│   │   ├── SettingsPanel.tsx
│   │   ├── AgentList.tsx
│   │   └── TrayPopup.tsx
│   ├── stores/
│   │   ├── agentStore.ts
│   │   ├── layoutStore.ts
│   │   ├── settingsStore.ts
│   │   └── trayStore.ts
│   ├── hooks/
│   │   └── useGhostty.ts
│   ├── types/
│   │   └── index.ts
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css
├── public/
│   └── wasm/
│       └── ghostty.webcodecs.mjs
├── index.html
├── vite.config.ts
└── tsconfig.json
```

#### Step 0.6: Rust Core Structure

```
src/
├── main.rs
├── agent/
│   ├── mod.rs
│   ├── launcher.rs
│   ├── session.rs
│   └── profile.rs
├── layout/
│   ├── mod.rs
│   ├── tree.rs
│   └── preset.rs
├── tray/
│   ├── mod.rs
│   └── manager.rs
├── config/
│   ├── mod.rs
│   ├── loader.rs
│   └── schema.rs
├── window/
│   ├── mod.rs
│   └── manager.rs
├── notification/
│   ├── mod.rs
│   └── sender.rs
└── utils/
    ├── mod.rs
    └── path.rs
```

#### Verification

- [ ] `cargo build` succeeds
- [ ] `npm run build` in web/ succeeds
- [ ] Tauri dev server (`cargo tauri dev`) launches with web UI
- [ ] Ghostty WASM loads in webview (canvas visible)
- [ ] Terminal renders blank screen

---

### Phase 1: Core Foundation (v3–v5 days)

Implement the core IPC bridge, config system, and agent spawning.

#### Step 1.1: IPC Bridge

Define the Tauri IPC handlers:

```rust
// src/main.rs
use tauri::Manager;
use std::sync::Mutex;

struct AppState {
    agents: Mutex<Vec<AgentState>>,
    settings: Mutex<Settings>,
}

#[tauri::command]
async fn spawn_agent(app: tauri::AppHandle, profile_id: String) -> Result<AgentId, String> {
    // Load profile from YAML
    let profile = load_profile(&profile_id)?;
    // Spawn PTY
    let session = Session::spawn(profile)?;
    // Register with IPC
    let id = session.id();
    // Notify webview
    app.emit("agent-spawned", &id).unwrap();
    Ok(id)
}

#[tauri::command]
async fn write_to_agent(app: tauri::AppHandle, agent_id: String, data: Vec<u8>) -> Result<(), String> {
    // Forward to session PTY
    let session = get_session(&app, &agent_id)?;
    session.write(&data)?;
    Ok(())
}

#[tauri::command]
async fn get_agent_status(app: tauri::AppHandle, agent_id: String) -> Result<AgentStatus, String> {
    let session = get_session(&app, &agent_id)?;
    Ok(session.status())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Load config.toml
            let settings = ConfigLoader::load()?;
            app.manage(AppState {
                agents: Mutex::new(Vec::new()),
                settings: Mutex::new(settings),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            spawn_agent,
            write_to_agent,
            get_agent_status,
            // ... more handlers
        ])
        .run(tauri::generate_context!())
        .expect("Failed to run Tauri app");
}
```

#### Step 1.2: PTY Spawner (Linux)

```rust
// src/agent/session.rs
use rustix::pty::*;
use std::io::{Read, Write};
use std::process::Command;

pub struct Session {
    id: AgentId,
    pid: nix::unistd::Pid,
    master: File,
    process: Child,
    status: AgentStatus,
}

impl Session {
    pub fn spawn(profile: &AgentProfile) -> Result<Self, String> {
        // Open PTY
        let (master, slave_name) = openpty(None, None).map_err(|e| e.to_string())?;
        
        // Setup terminal attributes
        let mut attrs = termios::tcgetattr(&slave).unwrap();
        attrs.c_oflag &= !termios::OPOST; // Raw output
        termios::tcsetattr(&slave, termios::AttrSpec::Now, &attrs).unwrap();
        
        // Build command
        let mut cmd = Command::new(&profile.command);
        cmd.args(&profile.args)
           .envs(&profile.env)
           .stdin(ChildPipe::Fd(slave.as_raw_fd()))
           .stdout(ChildPipe::Fd(slave.as_raw_fd()))
           .stderr(ChildPipe::Fd(slave.as_raw_fd()));
        
        // Spawn
        let process = cmd.spawn().map_err(|e| e.to_string())?;
        
        Ok(Self {
            id: AgentId::new(),
            pid: process.id(),
            master,
            process,
            status: AgentStatus::Launching,
        })
    }
    
    pub fn write(&mut self, data: &[u8]) -> Result<(), String> {
        self.master.write_all(data).map_err(|e| e.to_string())
    }
    
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, String> {
        self.master.read(buf).map_err(|e| e.to_string())
    }
}
```

#### Step 1.3: Config System

```rust
// src/config/schema.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub log_level: LogLevel,
    pub terminal: TerminalSettings,
    pub agents: AgentsSettings,
    pub notifications: NotificationSettings,
    pub hotkeys: HotkeySettings,
    pub crash_reporting: CrashReportingSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TerminalSettings {
    pub font_family: String,
    pub font_size: f64,
    pub scrollback: usize,
    pub theme: TerminalTheme,
    pub cursor_style: CursorStyle,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            font_family: "JetBrains Mono".to_string(),
            font_size: 15.0,
            scrollback: 10000,
            theme: TerminalTheme::default(),
            cursor_style: CursorStyle::Block,
        }
    }
}

// Generate default config.toml
// Write to ~/.config/splatter/config.toml
```

#### Step 1.4: Web Layer — Agent List

```typescript
// web/src/components/AgentList.tsx
import { useAgentStore } from '../stores/agentStore';

export function AgentList() {
  const agents = useAgentStore(s => s.agents);

  return (
    <div className="flex flex-col">
      {agents.map(agent => (
        <div key={agent.id} className="flex items-center gap-2 p-2 hover:bg-white/5">
          {/* Status dot */}
          <div className={`w-2 h-2 rounded-full ${statusColor(agent.status)}`} />
          {/* Agent name */}
          <span>{agent.name}</span>
          {/* Duration */}
          <span className="text-gray-400 text-xs">
            {formatDuration(agent.createdAt)}
          </span>
        </div>
      ))}
    </div>
  );
}
```

#### Verification

- [ ] `cargo build` succeeds
- [ ] `npm run build` succeeds
- [ ] Tauri dev server launches
- [ ] Config.toml generated at ~/.config/splatter/config.toml
- [ ] Spawn agent command works (spawn bash, see shell)
- [ ] Write command sends input to PTY
- [ ] Agent status returned correctly
- [ ] Agent list renders in web UI

---

### Phase 2: Terminal Integration (v6–v10 days)

Integrate Ghostty-web with the PTY bridge, implement multi-pane layout.

#### Step 2.1: PTY Read Loop

```rust
// In session.rs — background read loop
pub async fn start_read_loop(&mut self, app: AppHandle) {
    let mut buf = [0u8; 4096];
    loop {
        match self.read(&mut buf) {
            Ok(n) if n > 0 => {
                // Forward to webview
                app.emit("agent-output", &AgentOutputEvent {
                    agent_id: self.id.clone(),
                    data: buf[..n].to_vec(),
                }).unwrap();
            }
            Ok(0) => {
                // EOF — agent exited
                self.status = AgentStatus::Done;
                app.emit("agent-exit", &self.id).unwrap();
                break;
            }
            Err(e) => {
                self.status = AgentStatus::Error;
                app.emit("agent-error", &self.id).unwrap();
                break;
            }
        }
    }
}
```

#### Step 2.2: Web Layer — PTY Bridge

```typescript
// web/src/hooks/useGhostty.ts
import { useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { emit, listen } from '@tauri-apps/api/event';

export function useGhostty(container: HTMLDivElement, cols: number, rows: number) {
  const termRef = useRef<Terminal | null>(null);

  useEffect(() => {
    // Initialize Ghostty
    init().then(() => {
      const term = new Terminal({ cols, rows });
      term.open(container);
      termRef.current = term;

      // Forward terminal input to PTY
      term.onData(data => {
        invoke('write_to_agent', { agentId: agentId, data: [...new TextEncoder().encode(data)] });
      });

      // Listen for PTY output from Rust
      listen('agent-output', (event: Payload<AgentOutputEvent>) => {
        term.write(new Uint8Array(event.payload.data));
      });

      // Listen for agent status changes
      listen('agent-status', (event: Payload<AgentStatusEvent>) => {
        // Update Zustand store
        setAgentStatus(event.payload.agent_id, event.payload.status);
      });
    });

    return () => termRef.current?.dispose();
  }, [cols, rows, agentId]);
}
```

#### Step 2.3: BSP Tree Layout Engine

```rust
// src/layout/tree.rs

#[derive(Debug, Clone)]
pub enum LayoutNode {
    Leaf(LeafNode),     // Terminal pane
    Split(SplitNode),   // Horizontal or vertical split
}

#[derive(Debug, Clone)]
pub struct LeafNode {
    pub agent_id: Option<AgentId>,
    pub rect: Rect,
}

#[derive(Debug, Clone)]
pub struct SplitNode {
    pub direction: SplitDirection,
    pub left: Box<LayoutNode>,
    pub right: Box<LayoutNode>,
    pub ratio: f64, // 0.0..1.0
}

#[derive(Debug, Clone)]
pub enum SplitDirection {
    Horizontal, // Split top/bottom (split down/up)
    Vertical,   // Split left/right (split right/left)
}

pub struct LayoutTree {
    root: LayoutNode,
    focused: LayoutNodeId,
}

impl LayoutTree {
    pub fn new() -> Self {
        Self {
            root: LayoutNode::Leaf(LeafNode::full_screen()),
            focused: LayoutNodeId::root(),
        }
    }

    pub fn split(&mut self, direction: SplitDirection, ratio: f64) -> LayoutNodeId {
        // Split the focused leaf node
        let leaf = self.focused_leaf().unwrap();
        let new_node = LayoutNode::Split(SplitNode {
            direction,
            left: Box::new(leaf.clone()),
            right: Box::new(LeafNode::new()),
            ratio,
        });
        *leaf = new_node;
        self.focused = LayoutNodeId::new_right_child();
        self.recompute_rects();
        self.focused
    }

    pub fn close(&mut self, node_id: LayoutNodeId) -> Result<(), String> {
        // Remove the node, expand sibling
        let parent = self.parent_of(node_id)?;
        let sibling = self.sibling_of(node_id)?;
        *parent.replace_sibling(sibling)?;
        self.recompute_rects();
        Ok(())
    }

    pub fn focus(&mut self, direction: FocusDirection) {
        // Navigate to adjacent node
        self.focused = self.next_in_direction(self.focused, direction);
    }

    pub fn focus_by_id(&mut self, id: LayoutNodeId) {
        self.focused = id;
    }

    pub fn zoom(&mut self, node_id: LayoutNodeId) {
        // Maximize the node to full screen, restore on unzoom
        // ...
    }

    pub fn recompute_rects(&mut self) {
        // Traverse tree, assign rectangles
        fn visit(node: &mut LayoutNode, rect: Rect) {
            match node {
                LayoutNode::Leaf(leaf) => {
                    leaf.rect = rect;
                }
                LayoutNode::Split(split) => {
                    match split.direction {
                        SplitDirection::Horizontal => {
                            let top_height = (rect.height as f64 * split.ratio) as u32;
                            visit(&mut split.left, Rect::new(rect.x, rect.y, rect.width, top_height));
                            visit(&mut split.right, Rect::new(rect.x, rect.y + top_height, rect.width, rect.height - top_height));
                        }
                        SplitDirection::Vertical => {
                            let left_width = (rect.width as f64 * split.ratio) as u32;
                            visit(&mut split.left, Rect::new(rect.x, rect.y, left_width, rect.height));
                            visit(&mut split.right, Rect::new(rect.x + left_width, rect.y, rect.width - left_width, rect.height));
                        }
                    }
                }
            }
        }
        visit(&mut self.root, Rect::full_screen());
    }
}
```

#### Step 2.4: React Multi-Pane Layout

```tsx
// web/src/components/Layout.tsx
import { useLayoutStore } from '../stores/layoutStore';

export function Layout() {
  const nodes = useLayoutStore(s => s.nodes);
  const focusedId = useLayoutStore(s => s.focused);

  function renderNode(node: LayoutNode) {
    if (node.type === 'leaf') {
      return (
        <div key={node.id} className="relative h-full overflow-hidden"
             style={{ display: focusedId === node.id ? 'flex' : 'flex' }}>
          <GhosttyTerminal agentId={node.agentId} rect={node.rect} />
        </div>
      );
    }
    if (node.type === 'split') {
      const isHorizontal = node.direction === 'horizontal';
      return (
        <div key={node.id} className={`flex ${isHorizontal ? 'flex-col' : 'flex-row'} h-full`}>
          {renderNode(node.left)}
          <div className="bg-gray-700" style={{ flex: isHorizontal ? node.ratio : node.ratio }} />
          {renderNode(node.right)}
        </div>
      );
    }
  }

  return <div className="h-screen">{renderNode(nodes.root)}</div>;
}
```

#### Verification

- [ ] PTY output forwarded to Ghostty terminal in real-time
- [ ] Terminal input forwarded to PTY in real-time
- [ ] Layout tree created with initial full-screen pane
- [ ] Split right creates two panes (50/50)
- [ ] Split down creates two panes (50/50)
- [ ] Close pane expands sibling
- [ ] Focus navigation works (arrow keys, Cmd+HJKL)
- [ ] Zoom/unzoom works
- [ ] No layout crash on extreme splits (10+ panes)

---

### Phase 3: Agent Awareness (v11–v15 days)

Implement agent lifecycle tracking, status dots, activity log, handoff context.

#### Step 3.1: Agent Status System

```rust
// src/agent/session.rs

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Launching,  // Process starting
    Idle,       // Ready, waiting for input
    Working,    // Producing output
    Blocked,    // Blocked by output limit or user
    Done,       // Exited cleanly
    Error,      // Crashed or errored
}

impl Session {
    pub fn update_status(&mut self, new_status: AgentStatus) {
        let old_status = std::mem::replace(&mut self.status, new_status);
        // Record transition
        self.status_history.push(StatusTransition {
            from: old_status,
            to: new_status.clone(),
            at: chrono::Utc::now(),
        });
        // Emit event to webview
        self.app_handle.emit("agent-status", &AgentStatusEvent {
            agent_id: self.id.clone(),
            status: new_status,
            timestamp: chrono::Utc::now(),
        }).unwrap();
    }
}
```

#### Step 3.2: Agent Output Monitoring

```rust
// src/agent/session.rs

impl Session {
    pub fn monitor_output(&mut self) {
        let output_bytes = self.output.len();
        let output_lines = self.output.lines().count();

        // Detect working vs idle
        if output_bytes > self.last_bytes {
            self.update_status(AgentStatus::Working);
        } else if self.last_bytes > 0 && self.status == AgentStatus::Working {
            // Output stopped — could be blocked or done
            // Check if process is still running
            if self.process.has_stopped() {
                self.update_status(AgentStatus::Done);
            } else {
                // Could be blocked by terminal output limit
                self.update_status(AgentStatus::Blocked);
            }
        }
        self.last_bytes = output_bytes;
    }
}
```

#### Step 3.3: Activity Log

```rust
// src/agent/activity.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub timestamp: DateTime<Utc>,
    pub event: ActivityEvent,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub entries: Vec<ActivityEntry>,
}

impl ActivityLog {
    pub fn add(&mut self, event: ActivityEvent) {
        self.entries.push(ActivityEntry {
            timestamp: chrono::Utc::now(),
            event,
        });
    }

    pub fn replay(&self) -> Vec<String> {
        // Return events for replay in UI
        self.entries.iter().map(|e| format!("{:?}: {:?}", e.timestamp, e.event)).collect()
    }
}
```

#### Step 3.4: Handoff Context Capture

```rust
// src/agent/handoff.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffContext {
    pub agent_id: AgentId,
    pub profile_id: String,
    pub last_output: String,
    pub status: AgentStatus,
    pub activity_log: ActivityLog,
    pub started_at: DateTime<Utc>,
    pub duration: Duration,
    pub output_bytes: usize,
    pub output_lines: usize,
    pub notes: Vec<String>,
}

impl Session {
    pub fn capture_handoff(&self) -> HandoffContext {
        HandoffContext {
            agent_id: self.id.clone(),
            profile_id: self.profile_id.clone(),
            last_output: self.output.last_output(1000), // Last 1000 chars
            status: self.status.clone(),
            activity_log: self.activity_log.clone(),
            started_at: self.started_at,
            duration: chrono::Utc::now() - self.started_at,
            output_bytes: self.output.len(),
            output_lines: self.output.lines().count(),
            notes: self.notes.clone(),
        }
    }
}
```

#### Verification

- [ ] Agent status transitions tracked correctly (launching → idle → working → done)
- [ ] Status events emitted to webview in real-time
- [ ] Activity log entries created for each status change
- [ ] Activity log persists across restarts (saved to ~/.config/splatter/agents/<id>.json)
- [ ] Handoff context captured on agent exit
- [ ] Agent list shows correct status colors
- [ ] Agent duration displayed correctly

---

### Phase 4: Tray, Notifications, Hotkeys (v16–v22 days)

Implement system tray, notifications, and global hotkeys.

#### Step 4.1: System Tray

```rust
// src/tray/manager.rs

pub struct TrayManager {
    tray: TrayIcon,
    status: TrayStatus,
}

impl TrayManager {
    pub fn new(app_handle: AppHandle) -> Result<Self, String> {
        // Create tray icon
        let tray = TrayIcon::new()
            .icon(app_handle.default_window_icon()?.clone())
            .tooltip("Splatter")
            .menu(TrayMenu::new()
                .item(&MenuItem::with_id("show", "Show")?)
                .item(&MenuItem::with_id("hide", "Hide")?)
                .separator()
                .item(&MenuItem::with_id("quit", "Quit")?)
            )?;

        tray.set_tooltip("Splatter: 0 agents")?;

        Ok(Self { tray, status: TrayStatus::Idle })
    }

    pub fn update_status(&mut self, status: TrayStatus) {
        self.status = status;
        let (icon, tooltip) = self.to_tray_icon_and_tooltip();
        self.tray.set_icon(icon).ok();
        self.tray.set_tooltip(tooltip).ok();
    }

    fn to_tray_icon_and_tooltip(&self) -> (Icon, String) {
        match self.status {
            TrayStatus::Idle => (Icon::default(), "Splatter: 0 agents".to_string()),
            TrayStatus::Active { working, done, blocked, error } => {
                let tooltip = format!(
                    "Splatter: {} working, {} done, {} blocked, {} error",
                    working, done, blocked, error
                );
                if error > 0 {
                    // Use red icon (if available)
                    (Icon::default(), tooltip)
                } else if blocked > 0 {
                    (Icon::default(), tooltip)
                } else if done > 0 {
                    (Icon::default(), tooltip)
                } else {
                    (Icon::default(), tooltip)
                }
            }
        }
    }
}
```

#### Step 4.2: Notifications

```rust
// src/notification/sender.rs

pub struct NotificationSender {
    config: NotificationSettings,
}

impl NotificationSender {
    pub fn send(&self, title: &str, body: &str) -> Result<(), String> {
        // Check if notifications are enabled
        if !self.config.enabled {
            return Ok(());
        }

        // Check focus state
        if self.config.focus_when_focused && self.is_focused() {
            return Ok(());
        }

        // Use D-Bus on Linux (zbus)
        #[cfg(target_os = "linux")]
        {
            let conn = zbus::Connection::session().map_err(|e| e.to_string())?;
            let msg = Message::new_signal(
                "/org/freedesktop/Notifications",
                "org.freedesktop.Notifications",
                "Notify",
            )?;
            // ... send notification
        }

        // Use notify-send on Linux as fallback
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("notify-send")
                .arg(title)
                .arg(body)
                .spawn()
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn is_focused(&self) -> bool {
        // Check if any Splatter window is focused
        // ...
    }
}
```

#### Step 4.3: Global Hotkeys

```rust
// src/hotkeys/manager.rs

pub struct HotkeyManager {
    hotkeys: HashMap<String, GlobalShortcut>,
}

impl HotkeyManager {
    pub fn new(app_handle: AppHandle, config: &HotkeySettings) -> Result<Self, String> {
        let mut manager = Self {
            hotkeys: HashMap::new(),
        };

        // Register navigation hotkeys
        manager.register(
            "nav-prev-pane",
            config.nav_prev_pane.clone(), // Hotkey like "Cmd+Shift+Up"
            || app_handle.emit("hotkey-nav-prev-pane", ()).unwrap(),
        )?;

        manager.register(
            "nav-next-pane",
            config.nav_next_pane.clone(),
            || app_handle.emit("hotkey-nav-next-pane", ()).unwrap(),
        )?;

        manager.register(
            "nav-cycle-pane",
            config.nav_cycle_pane.clone(),
            || app_handle.emit("hotkey-nav-cycle-pane", ()).unwrap(),
        )?;

        manager.register(
            "nav-focus-left",
            config.nav_focus_left.clone(),
            || app_handle.emit("hotkey-nav-focus-left", ()).unwrap(),
        )?;

        manager.register(
            "nav-focus-down",
            config.nav_focus_down.clone(),
            || app_handle.emit("hotkey-nav-focus-down", ()).unwrap(),
        )?;

        manager.register(
            "nav-focus-up",
            config.nav_focus_up.clone(),
            || app_handle.emit("hotkey-nav-focus-up", ()).unwrap(),
        )?;

        manager.register(
            "nav-focus-right",
            config.nav_focus_right.clone(),
            || app_handle.emit("hotkey-nav-focus-right", ()).unwrap(),
        )?;

        // ... more hotkeys
        Ok(manager)
    }

    fn register(
        &mut self,
        id: &str,
        hotkey: Hotkey,
        handler: impl Fn() + Send + Sync + 'static,
    ) -> Result<(), String> {
        let shortcut = GlobalShortcut::new(app_handle, hotkey, handler)
            .map_err(|e| e.to_string())?;
        self.hotkeys.insert(id.to_string(), shortcut);
        Ok(())
    }
}
```

#### Step 4.4: Agent Interrupt via Hotkey

```rust
// src/agent/manager.rs

#[tauri::command]
async fn interrupt_agent(app: tauri::AppHandle, agent_id: String) -> Result<(), String> {
    let session = get_session(&app, &agent_id)?;
    // Send SIGINT to process
    nix::sys::signal::kill(session.pid(), nix::sys::signal::SIGINT)
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

#### Verification

- [ ] System tray visible with tooltip showing agent counts
- [ ] Tray icon color changes based on status (green=working, gray=done, etc.)
- [ ] Tray menu responds to clicks (show, hide, quit)
- [ ] Notifications sent on agent blocked/done (when config allows)
- [ ] Notifications suppressed when app focused
- [ ] Global hotkeys registered system-wide
- [ ] Hotkey `Cmd+Shift+Up` navigates to previous pane
- [ ] Hotkey `Cmd+Shift+Down` navigates to next pane
- [ ] Hotkey `Cmd+Tab` cycles through panes
- [ ] Hotkey `Cmd+I` interrupts agent (sends SIGINT)
- [ ] Hotkey `Cmd+Enter` splits pane down
- [ ] Hotkey `Cmd+Shift+Enter` splits pane right
- [ ] Hotkey `Cmd+Z` zooms/unzooms pane
- [ ] Hotkey `Cmd+D` closes pane

---

### Phase 5: Multi-Window (v23–v28 days)

Implement multi-window support, monitor detection, window state persistence.

#### Step 5.1: Multi-Window Manager

```rust
// src/window/manager.rs

pub struct WindowManager {
    windows: HashMap<String, WindowHandle>,
}

impl WindowManager {
    pub fn create(&mut self, label: &str, options: WindowOptions) -> Result<(), String> {
        if self.windows.contains_key(label) {
            return Err(format!("Window '{}' already exists", label));
        }

        let window = AppHandle::create_window(
            label.to_string(),
            WebviewWindowBuilder::new(
                &self.app_handle,
                label.to_string(),
                WindowUrl::App("index.html".into()),
            )
            .title("Splatter")
            .inner_size(options.width, options.height)
            .position(options.x, options.y)
            .resizable(options.resizable)
            .build()
            .map_err(|e| e.to_string())?,
        );

        self.windows.insert(label.to_string(), window);
        Ok(())
    }

    pub fn get(&self, label: &str) -> Option<&WindowHandle> {
        self.windows.get(label)
    }

    pub fn close(&mut self, label: &str) -> Result<(), String> {
        if let Some(window) = self.windows.remove(label) {
            window.close()?;
        }
        Ok(())
    }
}
```

#### Step 5.2: Monitor Detection

```rust
// src/window/monitor.rs

pub struct MonitorManager {
    monitors: Vec<Monitor>,
}

impl MonitorManager {
    pub fn new(app_handle: AppHandle) -> Result<Self, String> {
        let monitors = app_handle
            .primary_monitor()
            .map(|m| {
                vec![m] // Simplified — need to get all monitors
            })
            .map_err(|e| e.to_string())?;

        Ok(Self { monitors })
    }

    pub fn find_monitor_by_name(&self, name: &str) -> Option<&Monitor> {
        self.monitors.iter().find(|m| m.name() == Some(name.to_string()))
    }

    pub fn find_monitor_at_position(&self, x: i32, y: i32) -> Option<&Monitor> {
        self.monitors.iter().find(|m| {
            let size = m.size();
            x >= m.position().x && x < m.position().x + size.width &&
            y >= m.position().y && y < m.position().y + size.height
        })
    }

    pub fn auto_layout_windows(&self, windows: &[WindowOptions]) -> Vec<Rect> {
        // Distribute windows across monitors
        // ...
        vec![]
    }
}
```

#### Step 5.3: Window State Persistence

```rust
// In config.rs — save window state

pub fn save_window_state(&self, label: &str, position: Position, size: Size) -> Result<(), String> {
    let state = WindowState {
        position,
        size,
        maximized: false,
        fullscreen: false,
    };
    let mut map = self.window_states.write().unwrap();
    map.insert(label.to_string(), state);
    self.save()?; // Persist to config.toml
}

pub fn load_window_state(&self, label: &str) -> Option<WindowState> {
    let map = self.window_states.read().unwrap();
    map.get(label).cloned()
}
```

#### Verification

- [ ] Multiple windows created with unique labels
- [ ] Window created on correct monitor
- [ ] Window position/size restored on restart
- [ ] Window close emits event (not destroy)
- [ ] Monitor hotplug handled (new monitor detected)
- [ ] Multi-window auto-layout works (split across monitors)

---

### Phase 6: Settings UI + Plugins + Updater (v29–v35 days)

#### Step 6.1: Settings UI

```tsx
// web/src/components/SettingsPanel.tsx
import { useSettingsStore } from '../stores/settingsStore';

export function SettingsPanel() {
  const settings = useSettingsStore(s => s.settings);
  const updateSettings = useSettingsStore(s => s.update);

  return (
    <div className="flex h-full">
      {/* Sidebar */}
      <div className="w-48 border-r border-white/10 p-2">
        <div className="text-xs font-medium text-gray-400 uppercase mb-2">General</div>
        <button onClick={() => setActiveTab('general')}>General</button>
        <button onClick={() => setActiveTab('terminal')}>Terminal</button>
        <button onClick={() => setActiveTab('agents')}>Agents</button>
        <button onClick={() => setActiveTab('notifications')}>Notifications</button>
        <button onClick={() => setActiveTab('hotkeys')}>Hotkeys</button>
        <button onClick={() => setActiveTab('crash')}>Crash & Reporting</button>
      </div>

      {/* Content */}
      <div className="flex-1 p-4">
        {activeTab === 'terminal' && (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-300">Font Family</label>
              <select
                value={settings.terminal.font_family}
                onChange={e => updateSettings({ terminal: { ...settings.terminal, font_family: e.target.value } })}
              >
                <option value="JetBrains Mono">JetBrains Mono</option>
                <option value="Fira Code">Fira Code</option>
                <option value="Iosevka">Iosevka</option>
                <option value="monospace">monospace</option>
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300">Font Size</label>
              <input
                type="number"
                value={settings.terminal.font_size}
                onChange={e => updateSettings({ terminal: { ...settings.terminal, font_size: parseFloat(e.target.value) } })}
                min={8}
                max={32}
                step={1}
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300">Scrollback</label>
              <input
                type="number"
                value={settings.terminal.scrollback}
                onChange={e => updateSettings({ terminal: { ...settings.terminal, scrollback: parseInt(e.target.value) } })}
                min={1000}
                max={100000}
                step={1000}
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
```

#### Step 6.2: Plugin Host

```rust
// src/plugin/host.rs

pub struct PluginHost {
    plugins: HashMap<String, PluginInstance>,
}

#[derive(Debug, Clone)]
pub struct PluginInstance {
    pub name: String,
    pub version: String,
    pub manifest: PluginManifest,
    pub enabled: bool,
    pub state: PluginState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub entry: String, // JS file path
    pub permissions: Vec<String>,
    pub api: PluginApiVersion,
    pub scripts: PluginScripts, // "onReady", "onAgentStatusChanged"
}

impl PluginHost {
    pub fn load(&mut self, manifest_path: &Path) -> Result<(), String> {
        let manifest: PluginManifest = serde_json::from_str(
            &std::fs::read_to_string(manifest_path)?
        )?;

        // Validate version
        if manifest.api != PluginApiVersion::V1 {
            return Err(format!("Unsupported API version: {}", manifest.api));
        }

        // Validate permissions
        for perm in &manifest.permissions {
            if !ALLOWED_PERMISSIONS.contains(perm.as_str()) {
                return Err(format!("Permission '{}' not allowed", perm));
            }
        }

        // Create instance
        let instance = PluginInstance {
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            manifest,
            enabled: true,
            state: PluginState::Loading,
        };

        // Load JS entry point
        let entry_path = self.plugins_dir.join(&instance.manifest.entry);
        let js_code = std::fs::read_to_string(&entry_path)?;

        // ... execute JS in Tauri webview
        // ... handle plugin lifecycle

        self.plugins.insert(instance.name.clone(), instance);
        Ok(())
    }

    pub fn on_agent_status_changed(&self, event: &AgentStatusEvent) {
        for (_, plugin) in &self.plugins {
            if plugin.manifest.scripts.on_agent_status_changed {
                // Notify plugin of agent status change
                self.notify_plugin(&plugin.name, "onAgentStatusChanged", &event);
            }
        }
    }
}
```

#### Step 6.3: Auto-Updater Integration

```rust
// src/updater.rs

pub async fn check_for_updates() -> Result<Option<Update>, String> {
    // Use Tauri's built-in updater
    let app = tauri::api::get_app_handle();
    let update = app.updater().check().await?;

    if let Some(update) = update {
        if update.version > app.version()? {
            return Ok(Some(update));
        }
    }

    Ok(None)
}

#[tauri::command]
async fn download_update() -> Result<(), String> {
    let app = tauri::api::get_app_handle();
    app.updater().download_and_install().await?;
    Ok(())
}
```

#### Step 6.4: Crash Reporting (Sentry)

```rust
// In main.rs

fn main() {
    // Sentry integration (only when crash reporting enabled)
    let sentry_enabled = ConfigLoader::load().map(|c| c.crash_reporting.enabled).unwrap_or(false);

    if sentry_enabled {
        let _guard = sentry::init((
            "https://xxx@xxx.ingest.sentry.io/xxx",
            sentry::ClientOptions {
                release: sentry::release_name!(),
                debug: true,
                ..Default::default()
            }
        ));
    }

    tauri::Builder::default()
        // ...
}
```

#### Verification

- [ ] Settings UI renders all tabs
- [ ] Settings changes apply to terminal font/size/scrollback
- [ ] Settings persist across restarts
- [ ] Plugin manifest parsed correctly
- [ ] Plugin load fails gracefully on invalid manifest
- [ ] Plugin lifecycle hooks fire (onReady, onAgentStatusChanged)
- [ ] Plugin permissions enforced (HTTP requests blocked if no permission)
- [ ] Auto-updater checks for updates
- [ ] Crash reporting captures errors when enabled
- [ ] Crash reporting disabled by default

---

## Phase 7: Polish, Testing, Release (v36–v45 days)

### Step 7.1: E2E Testing

```bash
npm run test:e2e
```

Run Playwright tests:

- Agent spawn flow
- Multi-pane layout
- Hotkey navigation
- Agent status transitions
- Settings UI
- Tray interactions

### Step 7.2: Linux Packaging

```bash
cargo tauri build
```

Generate:

- `.deb` package (Debian/Ubuntu)
- `.AppImage` (universal Linux)

### Step 7.3: Distribution

- GitHub Releases with `.deb`, `.AppImage`, and `.tar.gz`
- Manual AUR PKGBUILD (community-maintained)
- Homebrew tap (brew tap)

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Agent spawn time | < 1 second |
| Layout ops (split/merge/close) | < 50 ms |
| Window create | < 500 ms |
| Memory (idle, 1 pane) | < 200 MB |
| Input latency | < 16 ms |
| Output latency | < 16 ms |
| WASM init | < 200 ms |

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Ghostty-web not yet stable | Terminal rendering breaks | Pin specific ghostty-web commit; maintain local fork |
| Tauri 2 Linux WebKitGTK issues | Terminal rendering broken on Linux | Test early; use latest WebKitGTK; have fallback renderer |
| Multi-monitor detection unreliable | Wrong window placement | Fallback to primary monitor; manual window placement |
| Plugin JS execution security | Sandbox escape | Use Tauri's built-in JS isolation; minimal permissions |
| PTY buffer overflow | Crash | Limit output buffer size; drop old data when full |
| Hotkey conflicts | Wrong hotkey fires | Warn on conflict; allow custom re-binding |

## Go/No-Go Checkpoints

### Phase 0 Gate (Day 2)

- [ ] Tauri 2 dev server running
- [ ] Ghostty WASM loaded in webview
- [ ] Basic React UI renders

### Phase 1 Gate (Day 5)

- [ ] Agent spawned in PTY
- [ ] PTY input/output working
- [ ] Config.toml loading/saving

### Phase 2 Gate (Day 10)

- [ ] Multi-pane layout working
- [ ] Split/close/zoom working
- [ ] Terminal rendering in each pane

### Phase 3 Gate (Day 15)

- [ ] Agent status transitions tracking
- [ ] Activity log maintained
- [ ] Handoff context captured

### Phase 4 Gate (Day 22)

- [ ] System tray functional
- [ ] Notifications working (Linux)
- [ ] Global hotkeys firing

### Phase 5 Gate (Day 28)

- [ ] Multi-window working
- [ ] Monitor detection working
- [ ] Window state persistence

### Phase 6 Gate (Day 35)

- [ ] Settings UI complete
- [ ] Plugin host functional
- [ ] Auto-updater ready

### Phase 7 Gate (Day 45)

- [ ] E2E tests passing
- [ ] `.deb`/`.AppImage` building
- [ ] Release ready
