# Splatter — Full Project Audit Report

**Date:** 2026-07-09
**Scope:** All layers — Rust core, Tauri 2 bindings, React 19 web frontend, Ghostty WASM integration
**Status:** 24 bugs/defects found across 6 categories

---

## 🔴 CRITICAL — Breaks Core Functionality

### C1. WASM Loading — "Could not connect to localhost: Connection refused"

**Severity:** P0 — App crashes on startup if WASM can't load
**Location:** `ghostty-web` → `useGhostty.ts` → `Ghostty.load()` fetch chain
**Root Cause:** The Ghostty WASM loader tries these fetch paths in order:

1. `file://` path (Node.js, fails in Tauri webview)
2. `import.meta.url` relative → `tauri://localhost/assets/ghostty-vt.wasm` (file doesn't exist there)
3. `./ghostty-vt.wasm` → relative (fails)
4. `/ghostty-vt.wasm` → but this hits Tauri's CSP or dev-server mismatch

In **dev mode**: The Vite dev server serves on `http://localhost:5173`, but the Ghostty loader in the webview tries to resolve `/ghostty-vt.wasm` relative to the Tauri webview URL. The `configureServer` middleware fix handles this for dev, but production Tauri build may not serve `ghostty-vt.wasm` from `dist/` root correctly.

**Fix:** Add explicit `wasmPath` to `Ghostty.load()` using Tauri's asset resolution, or configure Tauri to proxy the WASM path.

### C2. Layout Data Structure Mismatch — Flat Array vs Tree

**Severity:** P0 — Entire layout rendering broken
**Location:** `layout_commands.rs::get_layout()` ↔ `Layout.tsx` ↔ `layoutStore.ts`
**Root Cause:** `get_layout()` returns a **flat JSON array** of leaf panes:

```json
[{"id": 1, "rect": {...}, "agent_id": "xxx"}, {"id": 2, ...}]
```

But `Layout.tsx` expects a **tree structure** with nested `split`/`leaf` nodes:

```ts
{ type: "split", left: { type: "leaf" }, right: { type: "leaf" } }
```

The `setRoot()` call in `Layout.tsx` and `App.tsx` stores the flat array as `root`, then `renderNode()` can't find `node.type === "leaf"` or `"split"`.

**Fix:** Either:
A) Change `get_layout()` to return a proper BSP tree (recommended), OR
B) Change frontend to work with flat pane array

### C3. Ghostty `writeInput` Sends to Display Instead of PTY

**Severity:** P1 — User keyboard input never reaches PTY
**Location:** `useGhostty.ts` → `writeInput()`
**Code:**

```ts
const writeInput = useCallback((data: Uint8Array) => {
    termRef.current.write(data);  // ❌ writes to terminal DISPLAY, not PTY
}, []);
```

Should use `term.input(data, true)` or trigger the `onData` emitter to send to PTY.

### C4. Ghostty `onData` Callback Misnamed/Confused

**Severity:** P1 — PTY input flow unclear
**Location:** `GhosttyTerminal.tsx` → `useGhostty({ onOutput: ... })`
The `onOutput` callback in `useGhostty` actually fires on **user keyboard input** (Ghostty's `onData` event fires when user types), not terminal output. The callback name is misleading and the flow sends keyboard data to PTY via `write_to_agent`, which is correct but confusingly named.

---

## 🟠 HIGH — Significant Feature Breakage

### H1. Layout Store Not Synced with Tauri Backend

**Location:** `layoutStore.ts` ↔ `main.rs` layout state
**Issue:** The frontend maintains its own `panes: Map<number, Pane>` that is never synchronized with the actual `LayoutTree` in Rust. Splits, closes, and panes created in one place don't reflect in the other.

### H2. `splitPane()` Creates Duplicates

**Location:** `layoutStore.ts::splitPane()`
**Issue:** Both `left` and `right` leaf nodes get the same `rightNodeId`, and both panes map to the same `rightNodeId` in `newPanes`. Creates orphaned nodes and duplicate IDs.

### H3. Layout Tree Not Persisted Across Commands

**Location:** `layout_commands.rs::set_preset()`
**Issue:** `set_preset()` creates a new `LayoutTree` but doesn't set it on the `AppState.layout` — it just drops the value on the floor.

### H4. Ghostty Terminal Doesn't Get Focus

**Location:** `GhosttyTerminal.tsx`
**Issue:** The container div has `tabIndex={0}` but Ghostty's `open()` adds its own `contenteditable` textarea for input. When user clicks a terminal, focus goes to the parent div, not the Ghostty textarea, causing input to not reach the PTY.

### H5. Agent Spawn Race Condition

**Location:** `App.tsx` → `new_pane()` call
**Issue:** `invoke("new_pane", ...)` is called inside a `useEffect` without proper dependency management. If React StrictMode double-mounts, two agents spawn. Also, the agent store is updated before the `agent-spawned` event fires, creating potential state inconsistency.

---

## 🟡 MEDIUM — UX/Code Quality

### M1. Settings — Hotkeys Section Not Wrapped

**Location:** `Settings.tsx`
**Issue:** The Hotkeys section (`Object.entries(settings.hotkeys)`) is not wrapped in a `<SettingsSection>`, breaking visual consistency.

### M2. Settings — Copy-Paste Errors with Plugins Sections

**Location:** `Settings.tsx`
**Issue:** Four commented-out `<SettingsSection title="Plugins">` blocks litter the code:

```jsx
{/* Plugins Section }*
    <SettingsSection title="Plugins">...</SettingsSection>
```

These are dead code and indicate copy-paste errors.

### M3. Ghostty `onResize` Double-Fired

**Location:** `useGhostty.ts`
**Issue:** `onResize` is called both during initialization AND on `term.onResize()`. If Ghostty fires an initial resize event, `onResize` fires twice.

### M4. Ghostty Terminal Resize Calculation Inaccurate

**Location:** `GhosttyTerminal.tsx` → `cols/rows calculation`
**Code:**

```ts
cols: Math.max(10, Math.floor(rect.width / 8)),
rows: Math.max(3, Math.floor(rect.height / 16)),
```

Using fixed pixel-to-cell ratios (8px/col, 16px/row) doesn't match Ghostty's actual font sizing. When font size changes (e.g., 15px), the calculation is wrong.

### M5. Layout Tree Node IDs Use `Date.now()`

**Location:** `layoutStore.ts::splitPane()` and `setPreset()`
**Issue:** `Date.now()` generates numbers that can collide under rapid operations. Should use a counter or UUID.

### M6. `spawn_agent` Tauri Command Registered but Never Called

**Location:** `main.rs` → `agent_commands.rs`
**Issue:** The `spawn_agent` command is registered but `App.tsx` calls `new_pane` directly. Dead code.

---

## 🔵 LOW — Code Quality/Style

### L1. Unused Code

- `agent_commands.rs::spawn_agent()` — unused
- `hotkey/mod.rs` — 5 unused methods in `HotkeyConfig`
- `layout/mod.rs::next_in_direction()` — unused
- `layout_commands.rs::set_preset()` — doesn't actually apply the preset

### L2. Clippy Warnings

- `unsafe { nix::unistd::close(slave_fd) }` — unnecessary unsafe block (Line 214)
- `write_plugin(&PathBuf)` — should be `&Path`
- `profiles for the non root package will be ignored` — Cargo.toml config issue

### L3. Missing Error Boundaries

**Location:** `main.tsx`
**Issue:** No React error boundary — a single JS error crashes the entire app UI.

---

## 📊 Summary

| Category | Count | Impact |
|----------|-------|--------|
| 🔴 Critical (C) | 4 | Core functionality broken |
| 🟠 High (H) | 5 | Major feature gaps |
| 🟡 Medium (M) | 6 | UX/Quality issues |
| 🔵 Low (L) | 3 | Code hygiene |
| **Total** | **18** | |

**Estimated Fix Time:** 2–3 days (focused sprints)

---

## 🏗️ Recommended Fix Phases

### Phase 1: Layout Bridge (Day 1 AM)

- Fix `get_layout()` to return proper BSP tree
- Sync frontend layout with backend LayoutTree

### Phase 2: Ghostty WASM Loading (Day 1 PM)  

- Fix WASM path resolution for production Tauri
- Test in both dev and release modes

### Phase 3: Ghostty Input/Output (Day 2 AM)

- Fix `writeInput` to use `term.input()`
- Clarify `onData` ↔ PTY flow
- Fix focus behavior

### Phase 4: Layout Store Cleanup (Day 2 PM)

- Fix duplicate node IDs
- Remove `splitPane()` from store (use Tauri `new_pane`/`split_pane`)
- Remove commented-out code

### Phase 5: Build & Verify (Day 3)

- Full release build
- Desktop integration test
- E2E validation
