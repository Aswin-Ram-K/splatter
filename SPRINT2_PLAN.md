# Splatter — Next Sprint Plan (Sprint 2: Core Integration)

## Sprint Goal

Transform the scaffolding into a **working terminal multiplexer** — real Ghostty WASM terminal rendering, real PTY I/O bridge, multi-pane BSP layout, and system tray with notifications.

**Timeline:** 14 working days (Phases 8–11)
**Target:** Linux desktop app that can spawn agents, render real terminal output, and supports basic multi-pane layout.

---

## Current State Assessment

| Component | Status | Notes |
|-----------|--------|-------|
| Rust Core | ✅ Compiles, 13/13 tests | Agent manager, layout tree, config, notification shell, hotkey shell, plugin shell, crash shell, window shell, utils shell |
| Tauri 2 App | ✅ Compiles | IPC commands for agents/layout/config, Arc<Mutex<T>> state, Tauri CLI scaffolding |
| React Frontend | ✅ Compiles, 64KB gzipped | Layout renderer, AgentList, StatusBar, Zustand stores, Ghostty terminal wrapper (mock) |
| Ghostty WASM | ⚠️ Mock only | Type declarations exist, placeholder terminal renders nothing |
| PTY Bridge | ⚠️ Shell only | `write_to_agent` exists but doesn't connect to real PTY |
| System Tray | ⚠️ Shell only | TrayManager struct exists, no Tauri integration |
| Notifications | ⚠️ Shell only | NotificationSender struct exists, no D-Bus implementation |

---

## Phase 8: Ghostty WASM Integration (Days 1–4)

### Objective

Replace the mock terminal with real Ghostty WASM that renders VT output and captures keyboard input.

### 8.1 — Ghostty WASM Build & Link (Day 1)

**Tasks:**

1. Clone `coder/ghostty-web` and build WASM module
2. Build `ghostty.wasm` with WebCodecs backend (best Linux support)
3. Copy WASM + JS loader to `web/public/wasm/`
4. Configure Tauri to serve WASM assets from `wasm/` directory

**Implementation:**

```bash
# Clone and build ghostty-web
git clone --depth 1 https://github.com/coder/ghostty-web.git /tmp/ghostty-web
cd /tmp/ghostty-web

# Build WASM with WebCodecs backend
# (See ghostty-web docs for build steps — typically `just wasm` or `cargo build`)

# Copy artifacts
cp /tmp/ghostty-web/build/ghostty.wasm web/public/wasm/
cp /tmp/ghostty-web/build/ghostty.mjs web/public/wasm/
```

**File changes:**

- `web/public/wasm/ghostty.wasm` (binary)
- `web/public/wasm/ghostty.mjs` (JS loader)
- `web/src/hooks/useGhostty.ts` (update to import from real WASM)

**Testing:**

| Test | Type | Description |
|------|------|-------------|
| `wasm_loads_in_tauri` | Integration | WASM loads in Tauri webview, no console errors |
| `wasm_version_match` | Unit | `ghostty --version` matches expected version |
| `wasm_memory_limit` | Integration | WASM doesn't exceed memory limit during init |

### 8.2 — Real Terminal Initialization (Day 2)

**Tasks:**

1. Update `useGhostty.ts` to import and initialize real Ghostty WASM
2. Proper `init()` call with WebCodecs backend
3. Terminal `open()` into container div
4. Terminal `dispose()` on cleanup

**Implementation (`web/src/hooks/useGhostty.ts`):**

```typescript
import { init, Terminal, type ITerminalOptions } from 'ghostty-web';

export function useGhostty({ cols, rows }: { cols: number; rows: number }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    let term: Terminal;

    (async () => {
      try {
        await init({}); // Load WASM (idempotent)
        const termOptions: ITerminalOptions = {
          cols,
          rows,
          scrollback: 10000,
          cursorBlink: true,
          cursorStyle: 'block',
          fontFamily: '"JetBrains Mono", "Fira Code", monospace',
          fontSize: 15,
          theme: {
            background: '#1a1b26',
            foreground: '#a9b1d6',
            // ... full theme
          },
        };
        term = new Terminal(termOptions);
        term.open(containerRef.current);
        termRef.current = term;
      } catch (err) {
        console.error('Ghostty init failed:', err);
      }
    })();

    return () => {
      term?.dispose();
      termRef.current = null;
    };
  }, []);

  return { termRef };
}
```

**Testing:**

| Test | Type | Description |
|------|------|-------------|
| `terminal_opens_in_div` | Unit | Ghostty opens into container div, canvas element exists |
| `terminal_initial_size` | Unit | Terminal initialized with correct cols/rows |
| `terminal_resizes_on_reinit` | Unit | Terminal re-opens with new size on cols/rows change |
| `terminal_disposes_on_unmount` | Unit | dispose() called on unmount, no memory leak |

### 8.3 — Terminal Output Bridge (Day 3)

**Tasks:**

1. Rust side: spawn a real PTY process (e.g., `bash`) with output read loop
2. Read loop forwards PTY output via Tauri event → `agent-output`
3. Frontend side: `useGhostty` listens for `agent-output` events → calls `term.write()`
4. Test with `echo "Hello, World!"`

**Implementation (Rust — `splatter-core/src/agent/mod.rs`):**

```rust
pub async fn start_output_loop(&mut self, app_handle: AppHandle) {
    let app = app_handle.clone();
    let id = self.id.clone();
    
    tokio::spawn(async move {
        let mut buf = [0u8; 4096];
        loop {
            match self.read(&mut buf) {
                Ok(n) if n > 0 => {
                    app.emit("agent-output", AgentOutputEvent {
                        agent_id: id.to_string(),
                        data: buf[..n].to_vec(),
                    }).ok();
                }
                Ok(0) => {
                    app.emit("agent-exit", id.to_string()).ok();
                    break;
                }
                Err(e) => {
                    app.emit("agent-error", AgentErrorEvent {
                        agent_id: id.to_string(),
                        error: e.to_string(),
                    }).ok();
                    break;
                }
            }
        }
    });
}
```

**Implementation (Frontend — `web/src/hooks/useGhostty.ts`):**

```typescript
import { listen } from '@tauri-apps/api/event';

// Inside useEffect:
listen('agent-output', (event: Payload<{ agent_id: string; data: number[] }>) => {
  const term = termRef.current;
  if (term) {
    term.write(new Uint8Array(event.payload.data));
  }
});
```

**Testing:**

| Test | Type | Description |
|------|------|-------------|
| `pty_echo_hello` | Integration | Launch bash, write `\n`, verify "Hello, World!" renders |
| `pty_echo_ls` | Integration | Write `ls /\n`, verify directory listing renders |
| `pty_binary_output` | Integration | Write VT escape sequences, verify no crash |
| `pty_large_output` | Integration | Write 10K lines, verify no OOM or freeze |
| `pty_exit_detection` | Integration | Agent exits, `agent-exit` event fires |

### 8.4 — Keyboard Input Bridge (Day 4)

**Tasks:**

1. Ghostty terminal `onData()` → Tauri invoke `write_to_agent()`
2. Tauri side forwards input to PTY master
3. Test: type `echo test`, press Enter, see output

**Implementation (Frontend):**

```typescript
term.onData((data: string) => {
  invoke('write_to_agent', {
    agent_id: agentId,
    data: new TextEncoder().encode(data),
  });
});
```

**Implementation (Rust):**

```rust
#[tauri::command]
pub async fn write_to_agent(
    app: AppHandle,
    agent_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    let mut agents = app.state::<Arc<Mutex<AgentManager>>>().inner().lock().map_err(|e| e.to_string())?;
    let session = agents.get_mut(&agent_id).ok_or("Agent not found")?;
    session.write(&data).map_err(|e| e.to_string())
}
```

**Testing:**

| Test | Type | Description |
|------|------|-------------|
| `keyboard_echo` | Integration | Type `echo hi`, verify output renders |
| `keyboard_ctrl_c` | Integration | Press Ctrl+C, verify Ctrl+C sent to PTY |
| `keyboard_arrows` | Integration | Arrow keys → VT sequences forwarded |
| `keyboard_special` | Integration | Tab, Enter, Backspace all work correctly |

---

## Phase 9: Multi-Pane Layout & PTY Integration (Days 5–8)

### Objective

Connect real PTY sessions to real terminal rendering across multiple panes, with BSP layout.

### 9.1 — Real PTY Backend (Day 5)

**Tasks:**

1. Replace `AgentManager` mock PTY with real `rustix` PTY implementation
2. `openpty()` → master/slave pair
3. `child.stdin/stdout/stderr` → slave FD
4. Read loop on master FD (non-blocking with `select`/`poll`)

**Implementation (`splatter-core/src/agent/session.rs`):**

```rust
use rustix::pty::{openpty, Winsize};
use rustix::termios::*;
use std::os::fd::AsRawFd;

pub struct Session {
    pub id: AgentId,
    master_fd: OwnedFd,
    child: Child,
    status: AgentStatus,
}

impl Session {
    pub fn spawn(profile: &AgentProfile, cols: u16, rows: u16) -> Result<Self, String> {
        let mut termios = Termios::from_fd(&slave_fd)?;
        // Raw mode
        disable_output_processing(&mut termios)?;
        
        // Set window size
        let winsize = Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        ioctl::set_winsize(&slave_fd, &winsize)?;

        let mut cmd = Command::new(&profile.command);
        cmd.args(&profile.args)
           .stdin(File::from(slave_fd.try_clone()?))
           .stdout(File::from(slave_fd.try_clone()?))
           .stderr(File::from(slave_fd.try_clone()?));

        let child = cmd.spawn().map_err(|e| e.to_string())?;
        
        Ok(Self { id, master_fd, child, status: AgentStatus::Launching })
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut timeout = libc::pollfd {
            fd: self.master_fd.as_raw_fd(),
            events: POLLIN,
            revents: 0,
        };
        unsafe { libc::poll(&mut timeout as *mut _, 1, 10) };
        if timeout.revents & POLLIN != 0 {
            self.master_fd.read(buf)
        } else {
            Ok(0)
        }
    }

    pub fn write(&self, data: &[u8]) -> io::Result<usize> {
        self.master_fd.write(data)
    }
}
```

**Testing:**

| Test | Type | Description |
|------|------|-------------|
| `pty_open_success` | Unit | openpty() returns valid master/slave |
| `pty_raw_mode` | Unit | Process receives raw input (no echo) |
| `pty_window_size` | Unit | Process sees correct cols/rows via ioctl |
| `pty_child_spawn` | Unit | Child process visible in ps output |
| `pty_input_output` | Integration | Write to master, read from slave (via child) |

### 9.2 — PTY Read Loop with Tokio (Day 6)

**Tasks:**

1. Use `tokio` async read loop on PTY master FD
2. `tokio::io::AsyncFd` wrapping the master FD
3. Spawn per-agent read loop task
4. Buffer output, emit in chunks

**Implementation (`splatter-core/src/agent/mod.rs`):**

```rust
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

pub struct AgentManager {
    sessions: HashMap<AgentId, Session>,
    handles: HashMap<AgentId, tokio::task::JoinHandle<()>>,
}

impl AgentManager {
    pub async fn start_read_loop(&mut self, app_handle: AppHandle) {
        for (id, session) in self.sessions.iter_mut() {
            let app = app_handle.clone();
            let id = *id;
            
            let handle = tokio::spawn(async move {
                let mut master = tokio::io::AsyncFd::new(&session.master_fd).unwrap();
                let mut buf = [0u8; 4096];
                loop {
                    let n = master.get_mut().read(&mut buf).await.unwrap_or(0);
                    if n == 0 { break; }
                    app.emit("agent-output", AgentOutputEvent {
                        agent_id: id.to_string(),
                        data: buf[..n].to_vec(),
                    }).ok();
                }
            });
            self.handles.insert(id, handle);
        }
    }
}
```

**Testing:**

| Test | Type | Description |
|------|------|-------------|
| `tokio_read_loop_starts` | Unit | Read loop task spawned, no crash on init |
| `tokio_output_chunks` | Integration | Large output split into correct 4K chunks |
| `tokio_exit_detection` | Integration | Read loop exits on PTY EOF |
| `tokio_concurrent_agents` | Integration | 5 agents reading simultaneously, no interleaving |

### 9.3 — BSP Layout Multi-Pane (Day 7)

**Tasks:**

1. Frontend: Layout.tsx renders BSP tree with real Ghostty terminals
2. Each leaf node → `<GhosttyTerminal agentId="..." cols=... rows=... />`
3. BSP split creates two children with proportional widths/heights
4. Focus tracking: clicking a pane sets it as focused

**Implementation (`web/src/components/Layout.tsx`):**

```tsx
function renderNode(node: LayoutNode, agents: Map<string, AgentState>, onPaneClick: (id: string) => void) {
  if (node.type === 'leaf' && node.agent_id) {
    return (
      <GhosttyTerminal
        key={node.id}
        paneId={node.id}
        agentId={node.agent_id}
        agentState={agents.get(node.agent_id)}
        rect={node.rect || { x: 0, y: 0, width: 1280, height: 720 }}
        isFocused={true}
        onAgentSelect={onPaneClick}
      />
    );
  }
  if (node.type === 'split') {
    const isVertical = node.direction === 'vertical';
    return (
      <div className={`flex ${isVertical ? 'flex-row' : 'flex-col'} h-full`}>
        {node.left && renderNode(node.left, agents, onPaneClick)}
        <div className="bg-gray-700 w-[1px] shrink-0" />
        {node.right && renderNode(node.right, agents, onPaneClick)}
      </div>
    );
  }
  return null;
}
```

**Testing:**

| Test | Type | Description |
|------|------|-------------|
| `single_pane_render` | Integration | One pane renders with bash shell |
| `split_vertical` | Integration | Split vertical → two panes side by side |
| `split_horizontal` | Integration | Split horizontal → two panes top/bottom |
| `multi_pane_input` | Integration | Type in each pane independently |
| `multi_pane_output` | Integration | Each pane shows its own PTY output |
| `layout_resize` | Integration | Window resize → all panes resize correctly |

### 9.4 — Agent Spawn → Render Pipeline (Day 8)

**Tasks:**

1. AgentList "New Agent" button → Tauri spawn → PTY → terminal render
2. Agent spawns → Zustand store update → AgentList refreshes with new agent
3. Layout creates leaf → GhosttyTerminal mounts → PTY output appears
4. End-to-end: click "New Agent" → see bash shell in pane

**Testing:**

| Test | Type | Description |
|------|------|-------------|
| `spawn_agent_e2e` | Integration | Click "New Agent" → bash prompt renders in pane |
| `spawn_then_type` | Integration | Type `echo test` in pane → output renders |
| `spawn_two_agents` | Integration | Split, spawn agent in second pane, type independently |
| `close_pane` | Integration | Close pane → sibling expands, other panes unaffected |
| `focus_switch` | Integration | Click different panes → keyboard input goes to active pane |

---

## Phase 10: System Tray & Notifications (Days 9–11)

### Objective

Add system tray with agent counts, colored icon, and notification system.

### 10.1 — System Tray (Day 9)

**Tasks:**

1. Use `tauri-plugin-tray-icon` to create system tray
2. Tray icon with colored dot overlay (green/gray/red)
3. Tooltip with agent counts
4. Menu: Show, Quit

**Implementation (`splatter-core/src-tauri/src/main.rs`):**

```rust
use tauri::menu::{Menu, MenuItem, MenuBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

#[tauri::command]
pub async fn update_tray_status(app: AppHandle, status: TrayStatus) -> Result<(), String> {
    let (icon, tooltip) = match &status {
        TrayStatus { working: 0, done: 0, blocked: 0, error: 0 } => {
            // Default icon
            (None, format!("Splatter: {} agents", status.working + status.done + status.blocked + status.error))
        }
        _ => {
            let color = if status.error > 0 { "red" } 
                     else if status.working > 0 { "green" } 
                     else if status.blocked > 0 { "yellow" }
                     else { "gray" };
            (None, format!("Splatter: {} working, {} error", status.working, status.error))
        }
    };
    
    app.tray_by_id("main").ok().map(|tray| {
        tray.set_tooltip(&tooltip).ok();
    });
    
    Ok(())
}
```

**Testing:**

| Test | Type | Description |
|------|------|-------------|
| `tray_visible_on_launch` | Integration | Tray icon appears on app start |
| `tray_tooltip_count` | Integration | Tooltip shows correct agent count |
| `tray_color_change` | Integration | Icon color changes based on agent status |
| `tray_menu_quit` | Integration | Menu → Quit closes app |

### 10.2 — Notifications (Day 10)

**Tasks:**

1. Use `notify-send` CLI on Linux for notifications
2. Config flag to enable/disable
3. Auto-notify on agent done/blocked/error
4. Coalesce: max 1 notification per 5 seconds

**Implementation (`splatter-core/src/notification/mod.rs`):**

```rust
use std::process::Command;

pub struct NotificationSender {
    pub sound: bool,
    pub enabled: bool,
}

impl NotificationSender {
    pub fn send(&self, title: &str, body: &str) {
        if !self.enabled { return; }
        
        #[cfg(target_os = "linux")]
        {
            let mut cmd = Command::new("notify-send");
            cmd.arg("--app-name=Splatter");
            if self.sound {
                cmd.arg("-u", "normal");
            }
            cmd.arg(title).arg(body);
            cmd.spawn().ok();
        }
    }
}
```

**Testing:**

| Test | Type | Description |
|------|------|-------------|
| `notification_sends_linux` | Integration | `notify-send` called, notification appears |
| `notification_disabled` | Unit | When `enabled=false`, no notification sent |
| `notification_coalesce` | Integration | Two events within 5s → only one notification |
| `notification_on_agent_done` | Integration | Agent exits → notification sent |

### 10.3 — Tray-Notification Integration (Day 11)

**Tasks:**

1. Agent status event → update both tray and notification
2. Coalesce: tray updates always, notifications deduplicated
3. "New Agent" creates tray count update + optional notification
4. "Agent done" creates notification (if not focused)

---

## Phase 11: Polish & Integration Tests (Days 12–14)

### Objective

Fix edge cases, add integration tests, ensure everything works end-to-end.

### 11.1 — Edge Case Fixes (Day 12)

**Tasks:**

1. PTY buffer overflow → drop old output when buffer full
2. Terminal resize handling → `term.resize(cols, rows)`
3. Agent crash → proper error status, graceful cleanup
4. Window focus tracking → suppress notifications when app focused

### 11.2 — Integration Tests (Day 13)

**Tasks:**

1. Add Rust integration tests for PTY bridge
2. Add Vitest tests for layout rendering
3. Manual E2E test checklist
4. Performance test: 5 panes × bash, type simultaneously

### 11.3 — Polish & Documentation (Day 14)

**Tasks:**

1. README update with build/run instructions
2. Screenshots of working app
3. Known issues list
4. Prepare PR

---

## Test Plan — Sprint 2

### Rust Tests (Unit + Integration)

| # | Test | File | Type | Priority |
|---|------|------|------|----------|
| 1 | `pty_spawn_bash` | `agent/mod.rs` | Integration | P0 |
| 2 | `pty_spawn_error` | `agent/mod.rs` | Integration | P0 |
| 3 | `pty_write_read` | `agent/mod.rs` | Integration | P0 |
| 4 | `pty_exit_detect` | `agent/mod.rs` | Integration | P0 |
| 5 | `pty_resize` | `agent/mod.rs` | Integration | P1 |
| 6 | `pty_multi_concurrent` | `agent/mod.rs` | Integration | P1 |
| 7 | `layout_split_vertical` | `layout/mod.rs` | Unit | P0 |
| 8 | `layout_split_horizontal` | `layout/mod.rs` | Unit | P0 |
| 9 | `layout_close_pane` | `layout/mod.rs` | Unit | P0 |
| 10 | `layout_focus_nav` | `layout/mod.rs` | Unit | P1 |
| 11 | `layout_rect_assignment` | `layout/mod.rs` | Unit | P1 |
| 12 | `layout_preset_2x2` | `layout/mod.rs` | Unit | P2 |
| 13 | `notification_send_success` | `notification/mod.rs` | Integration | P0 |
| 14 | `notification_disabled` | `notification/mod.rs` | Unit | P0 |
| 15 | `notification_coalesce` | `notification/mod.rs` | Integration | P1 |

### Frontend Tests (Vitest)

| # | Test | File | Type | Priority |
|---|------|------|------|----------|
| 16 | `ghostty_init_success` | `useGhostty.test.ts` | Unit | P0 |
| 17 | `ghostty_write_output` | `useGhostty.test.ts` | Unit | P0 |
| 18 | `ghostty_on_data` | `useGhostty.test.ts` | Unit | P0 |
| 19 | `layout_render_single` | `Layout.test.tsx` | Unit | P0 |
| 20 | `layout_render_split` | `Layout.test.tsx` | Unit | P0 |
| 21 | `layout_render_nested` | `Layout.test.tsx` | Unit | P1 |
| 22 | `agentlist_filter` | `AgentList.test.tsx` | Unit | P1 |
| 23 | `agentlist_status_dots` | `AgentList.test.tsx` | Unit | P1 |
| 24 | `statusbar_counts` | `StatusBar.test.tsx` | Unit | P0 |

### Manual E2E Checklist

| # | Test | Steps | Pass |
|---|------|-------|------|
| 25 | Full pipeline | New Agent → bash prompt appears → type `echo test` → output renders | ☐ |
| 26 | Split pane | Cmd+Shift+R → two panes → type in each independently | ☐ |
| 27 | Close pane | Cmd+D → pane closes, sibling expands | ☐ |
| 28 | Resize window | Drag corner → all panes resize | ☐ |
| 29 | Focus switch | Click different panes → input goes to active | ☐ |
| 30 | Tray visible | Tray icon shows, tooltip has counts | ☐ |
| 31 | Notification | Agent done → notification appears | ☐ |

---

## Go/No-Go Checkpoints

### Phase 8 Gate (Day 4)

- [ ] Ghostty WASM loads in Tauri webview
- [ ] Terminal renders `echo test` output
- [ ] Keyboard input forwarded to PTY
- [ ] `cargo build` succeeds (0 errors, warnings acceptable)

### Phase 9 Gate (Day 8)

- [ ] Real PTY with bash renders shell prompt
- [ ] Split pane creates two independent terminals
- [ ] Each pane shows its own output
- [ ] Closing a pane works correctly

### Phase 10 Gate (Day 11)

- [ ] System tray shows agent counts
- [ ] Notifications fire on agent events
- [ ] `cargo build` succeeds (0 errors)

### Phase 11 Gate (Day 14)

- [ ] All 15 Rust tests pass
- [ ] All 9 Vitest tests pass
- [ ] All 7 manual E2E tests pass
- [ ] No memory leaks after 5-minute stress test

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Ghostty WASM build fails | Blocker | Pin known-good ghostty commit; use pre-built WASM if needed |
| PTY I/O hangs | High | Use `select()`/`poll()` with timeout; add 5s watchdog per PTY |
| WebKitGTK + WASM GPU crash | High | Fallback to Canvas backend if WebCodecs fails |
| Memory leak in PTY read loop | Medium | Add heap profile check; limit output buffer to 10MB |
| Multiple panes → keyboard routing broken | Medium | Track focused pane ID; forward input only to active terminal |

---

## Deliverables

1. **Working app** that spawns real agents with real terminal rendering
2. **Multi-pane layout** with independent PTY sessions per pane
3. **System tray** with live agent counts
4. **Notifications** on agent events
5. **Test suite**: 24 new tests (15 Rust + 9 Vitest + 7 manual E2E)
6. **Updated SKETCH.md** with implementation status
7. **Screenshots** of working app in PR
