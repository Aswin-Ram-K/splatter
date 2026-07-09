# Splatter — Build, Audit, Fix & Verify Plan

**Last updated:** 2026-07-09  
**Status:** Phase 0 (Audit & Stabilization) in progress

---

## Phase 0 — Audit & Stabilization (TODAY)

**Goal:** Ensure the app launches, loads from embedded dist, shows UI (not blank), and is fully isolated from para-site.

### 0.1 — Confirm release build is clean

- [ ] `cargo build --release` completes with zero errors
- [ ] `cargo test --package splatter-core --lib` — all 14 tests pass
- [ ] `web/src` TypeScript compiles with zero errors (`npx tsc --noEmit`)
- [ ] `web/dist` contains: `index.html`, `assets/index-*.js`, `assets/ghostty-*.js`, `ghostty-vt.wasm`
- [ ] Binary size ≤ 10MB (release, LTO, strip)

**Current status:** ✅ Pass (14 tests, 0 TS errors, ~9.5MB binary)

### 0.2 — Verify CSP & devUrl isolation

- [ ] `tauri.conf.json` CSP: NO `http://localhost:5173`
- [ ] `devUrl` ≠ para-site port (use `http://localhost:15173` or unset)
- [ ] Production build uses `frontendDist` (not `devUrl`) — confirmed via `tauri_protocol_url` → `tauri://localhost`
- [ ] Runtime: zero connections from WebKit to port 5173 (verified via `/proc/PID/net/tcp`)
- [ ] Runtime: zero connections from WebKit to port 15173

**Current status:** ✅ Pass (CSP clean, devUrl changed to 15173, no 5173 connections)

### 0.3 — Fix blank screen / "error" display

**Symptom:** App window opens but shows blank screen with an error message.

**Root cause hypotheses:**

1. **Ghostty WASM load failure** — `Ghostty.load("/ghostty-vt.wasm")` fails in Tauri webview → terminal doesn't render → blank pane
2. **React crash** — a runtime JS error in `App.tsx` or a component prevents rendering
3. **Tauri IPC failure** — `invoke()` calls fail silently → `new_pane` doesn't create any panes → blank layout
4. **CSS/layout issue** — Tailwind not compiling → no styles → white/blank screen

**Fix plan (execute sequentially, verify each):**

#### Fix 0.3a — Ensure Ghostty WASM is bundled in dist

- [ ] Verify `ghostty-vt.wasm` exists at `web/dist/ghostty-vt.wasm` (copied by Vite plugin)
- [ ] Verify `vite.config.ts` has `copyWasmPlugin` that copies WASM to `dist/`
- [ ] Verify `Ghostty.load("/ghostty-vt.wasm")` path is correct for Tauri `tauri://localhost` serving

**Code locations:**

- `web/vite.config.ts` — manual chunking + WASM copy plugin
- `web/src/hooks/useGhostty.ts` — `Ghostty.load("/ghostty-vt.wasm")`

#### Fix 0.3b — Add error boundary in React

- [ ] Wrap App in React Error Boundary component
- [ ] Display error message in webview so user can see what's crashing
- [ ] Log error to `console.error` and Tauri IPC

**Code locations:**

- `web/src/App.tsx` — add error boundary wrapper

#### Fix 0.3c — Verify Tauri IPC works

- [ ] Add `console.log` in `App.tsx` `invoke()` callbacks to verify IPC
- [ ] Check that `new_pane` returns an `agent_id`
- [ ] Verify `agent-spawned` event fires with correct payload
- [ ] Verify `get_layout` returns valid BSP tree

**Code locations:**

- `web/src/App.tsx` — `invoke("new_pane")` and `invoke("get_layout")`
- `web/src/stores/layoutStore.ts` — Zustand store
- `splatter-core/src-tauri/src/layout_commands.rs` — `new_pane`, `get_layout`

#### Fix 0.3d — Verify CSS loads (Tailwind)

- [ ] Check `web/dist/assets/index-*.css` exists and has content
- [ ] Verify `index.html` references the CSS file
- [ ] Verify Tailwind directives in `web/src/index.css` are processed

**Code locations:**

- `web/src/index.css` — `@tailwind` directives
- `web/vite.config.ts` — Tailwind plugin
- `web/dist/index.html` — CSS link tag

### 0.4 — Launch & visual test (manual)

- [ ] Build release: `cargo tauri build` or `cargo build --release` + Vite build
- [ ] Launch: `./target/release/splatter`
- [ ] Verify: Window shows Splatter UI (not blank, not para-site)
- [ ] Verify: Sidebar shows "AgentList"
- [ ] Verify: Main area shows Ghostty terminal (not empty)
- [ ] Verify: Status bar at bottom shows agent status
- [ ] Verify: No errors in console (enable WebKit inspector: `WEBKIT_INSPECTOR_SERVER=127.0.0.1:9999`)

**Visual checklist:**

- [ ] Dark theme background (#1a1b26)
- [ ] Left sidebar with agent list
- [ ] Terminal rendering with xterm-compatible chars
- [ ] Status bar at bottom with connection status

---

## Phase 1 — Ghostty Terminal Integration

**Goal:** Terminal renders correctly, accepts input, displays PTY output.

### 1.1 — Ghostty WASM loads and initializes

- [ ] `Ghostty.load("/ghostty-vt.wasm")` succeeds
- [ ] `Terminal` instance created
- [ ] `term.open(container)` renders to DOM
- [ ] Terminal shows xterm buffer (not blank)

**Test:** Type in terminal → expect visible characters

### 1.2 — PTY input/output bridge

- [ ] User types → `onInput()` callback fires → `invoke("write_to_agent")` sent
- [ ] Rust PTY read loop reads from slave FD → emits `agent-output` event
- [ ] `listen("agent-output")` receives data → `writeOutput()` feeds to Ghostty
- [ ] Terminal displays output in real-time

**Test:** Spawn agent → expect PTY output in terminal

### 1.3 — Terminal resize

- [ ] Window resize → `onResize()` fires
- [ ] `invoke("layout_resize")` sent to Rust
- [ ] Rust sends `TIOCSWINSZ` ioctl to PTY
- [ ] Terminal adjusts cols/rows

**Test:** Resize window → expect terminal to resize

---

## Phase 2 — Multi-Pane Layout

**Goal:** User can split panes, create new panes, close panes.

### 2.1 — BSP layout renders

- [ ] `invoke("get_layout")` returns BSP tree
- [ ] `Layout.tsx` renders tree as CSS grid/flex panes
- [ ] Each leaf pane has a `GhosttyTerminal` component

**Test:** Launch → expect single pane filling window

### 2.2 — Split pane

- [ ] User triggers split (hotkey or UI button)
- [ ] `invoke("split_pane", { pane_id, direction })` called
- [ ] Rust updates `LayoutTree` with new split node
- [ ] `layout-changed` event emitted
- [ ] Frontend re-renders layout with split panes

**Test:** Split horizontally/vertically → expect two panes

### 2.3 — Close pane

- [ ] User triggers close (hotkey or UI button)
- [ ] `invoke("close_pane", { pane_id })` called
- [ ] Rust removes leaf from `LayoutTree`
- [ ] Parent node promoted

**Test:** Close pane → expect remaining pane to fill space

---

## Phase 3 — Agent Management

**Goal:** Agents spawn, run, and can be managed from UI.

### 3.1 — Agent spawn

- [ ] `invoke("new_pane", { profile_id })` creates pane + spawns agent
- [ ] Agent appears in `AgentList` sidebar
- [ ] Agent status shows (idle, running, finished)

**Test:** Click "Spawn" → expect agent in list with output in terminal

### 3.2 — Agent interrupt

- [ ] User clicks interrupt button or presses hotkey
- [ ] `invoke("interrupt_agent", { agent_id })` called
- [ ] Rust sends `SIGINT` to agent process group

**Test:** Interrupt running agent → expect process to stop

### 3.3 — Agent pin / unpin

- [ ] `invoke("pin_agent", { agent_id })` / `invoke("unpin_agent", { agent_id })`
- [ ] Agent persists in pinned list
- [ ] Pinned agents shown at top of sidebar

**Test:** Pin an agent → expect it stays after restart

---

## Phase 4 — Plugin System

**Goal:** Plugins load from manifest, hooks fire correctly.

### 4.1 — Plugin manifest

- [ ] `plugin_manifest.yaml` in `~/.config/splatter/plugins/`
- [ ] Plugin host loads all plugins from manifest

**Test:** Place plugin manifest → expect plugin listed in Settings

### 4.2 — Plugin hooks

- [ ] `on_agent_output` hook fires when PTY output received
- [ ] `on_hotkey` hook fires for registered hotkeys
- [ ] `on_status_change` hook fires when agent status changes

**Test:** Trigger hook → expect plugin to receive event

### 4.3 — Plugin toggle

- [ ] Settings UI shows plugin list
- [ ] Toggle enable/disable plugin
- [ ] `toggle_plugin` command updates plugin state

**Test:** Disable plugin → expect it no longer receives events

---

## Phase 5 — Settings & Config

**Goal:** Settings persisted to `~/.config/splatter/config.toml`.

### 5.1 — Settings UI

- [ ] Settings modal opens/closes
- [ ] Terminal settings tab (font, size, theme)
- [ ] Agent settings tab (default profile, working dir)
- [ ] Hotkeys tab (rebindable keys)
- [ ] Plugins tab (plugin list with toggles)

**Test:** Open settings → expect tabs with controls

### 5.2 — Config persistence

- [ ] `save_config` command writes to TOML
- [ ] `get_config` command reads from TOML
- [ ] Config survives app restart

**Test:** Change setting → restart app → expect setting persisted

---

## Phase 6 — Notifications & Tray

**Goal:** Desktop notifications and system tray integration.

### 6.1 — Notifications

- [ ] `NotificationSender` fires notification on agent event
- [ ] System notification appears (GNOME/KDE)
- [ ] Settings allow notification config

**Test:** Agent finishes → expect notification

### 6.2 — System tray

- [ ] Tray icon shows (colored based on status)
- [ ] Tooltip shows agent counts
- [ ] Quick actions: open window, quit

**Test:** Launch app → expect tray icon

---

## Phase 7 — Packaging & Distribution

**Goal:** Build `.deb` and `.AppImage` for Linux.

### 7.1 — DEB package

- [ ] `cargo tauri build` produces `.deb`
- [ ] `.deb` installs correctly on Ubuntu/Debian
- [ ] Desktop shortcut created
- [ ] Icons embedded in package

**Test:** Install `.deb` → expect app in menu + desktop shortcut

### 7.2 — AppImage

- [ ] `cargo tauri build` produces `.AppImage`
- [ ] AppImage runs on any Linux distro
- [ ] Icon shows in AppImage

**Test:** Run `.AppImage` → expect app launches

### 7.3 — ISO (bonus)

- [ ] Custom ISO with Splatter pre-installed
- [ ] ISO boots to Splatter desktop

**Status:** Out of scope for v1

---

## Verification Checklist

### Build Verification

- [ ] `cargo build --release` ✅
- [ ] `cargo test --package splatter-core --lib` ✅ (14 tests pass)
- [ ] `web/src` TypeScript: zero errors ✅
- [ ] `web/dist` built ✅
- [ ] Binary ~9.5MB ✅

### Runtime Verification

- [ ] App launches ✅
- [ ] NOT connecting to para-site (port 5173) ✅
- [ ] NOT connecting to devUrl (port 15173) ✅
- [ ] Loading from embedded dist (`tauri://localhost`) ✅
- [ ] UI renders (not blank) ⏳ **NEEDS MANUAL TEST**
- [ ] Terminal renders ⏳ **NEEDS MANUAL TEST**
- [ ] Input works ⏳ **NEEDS MANUAL TEST**
- [ ] PTY output works ⏳ **NEEDS MANUAL TEST**
- [ ] Split pane works ⏳ **NEEDS MANUAL TEST**
- [ ] Agent spawn works ⏳ **NEEDS MANUAL TEST**
- [ ] Settings persist ⏳ **NEEDS MANUAL TEST**
- [ ] Notifications fire ⏳ **NEEDS MANUAL TEST**
- [ ] Tray icon shows ⏳ **NEEDS MANUAL TEST**
- [ ] `.deb` installs ⏳ **NEEDS MANUAL TEST**

---

## Files Modified (Recent)

| File | Change | Status |
|------|--------|--------|
| `splatter-core/src-tauri/tauri.conf.json` | CSP cleaned, devUrl changed to 15173 | ✅ |
| `web/src/hooks/useGhostty.ts` | WASM load fixed, fallback removed | ✅ |
| `web/src/components/Ghostty/GhosttyTerminal.tsx` | Terminal wrapper | ✅ |
| `web/src/App.tsx` | Error boundary + debug logs | 🔄 In progress |
| `web/tsconfig.json` | `skipLibCheck` for deprecated warning | ✅ |
| `web/src/vite-env.d.ts` | CSS type declarations | ✅ |
| `splatter-core/src/agent/mod.rs` | PTY read loop, drain_outputs | ✅ |
| `splatter-core/src/layout/mod.rs` | Vec-based BSP tree, presets | ✅ |
| `splatter-core/src-tauri/src/main.rs` | PTY background task | ✅ |
| `splatter-core/src-tauri/src/agent_commands.rs` | write_to_agent, list_agents | ✅ |
| `splatter-core/src-tauri/src/layout_commands.rs` | new_pane, split_pane, close_pane | ✅ |

---

## Known Issues

### Critical (Blocking)

1. **Blank screen / error display** — App launches but UI doesn't render. Need to enable WebKit inspector to see JS errors.
   - **Fix:** Enable `WEBKIT_INSPECTOR_SERVER=127.0.0.1:9999`, then open `chrome://inspect` to see JS errors
   - **OR:** Add React error boundary to display errors in the webview

### High

2. **Ghostty WASM load** — May fail if `/ghostty-vt.wasm` path doesn't resolve in `tauri://localhost`
   - **Fix:** Verify WASM is at `web/dist/ghostty-vt.wasm` and accessible via `tauri://localhost/ghostty-vt.wasm`

2. **Tailwind CSS** — May not compile if PostCSS config is missing
   - **Fix:** Verify `postcss.config.js` exists and `tailwindcss` is installed

### Medium

4. **Agent spawn** — May fail if `pi-agent` profile doesn't exist
   - **Fix:** Verify profile file exists at `resources/profiles/pi-agent.yaml`

2. **PTY read loop** — May hang if FD handling is incorrect
   - **Fix:** Verify `rustix::fd::OwnedFd` for PTY master/slave

3. **Layout sync** — Rust tree and React tree may desync
   - **Fix:** Ensure `get_layout` returns complete BSP tree after every change

---

## Git Commands to Track Progress

```bash
# After each phase, commit:
git add -A
git commit -m "Phase X: <description>"

# Push after approval:
git push origin main
```
