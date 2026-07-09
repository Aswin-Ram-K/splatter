# Splatter — Full Project Audit Report

**Date:** 2026-07-09  
**Scope:** Rust core (`splatter-core`), Tauri bindings (`src-tauri`), React frontend (`web`)  
**Status:** Compiles ✅, 14 tests pass ✅, but has **18 bugs** across all layers

---

## Severity Key

| Level | Meaning |
|-------|---------|
| 🔴 **Critical** | App won't render / crashes / data loss |
| 🟠 **High** | Feature broken, wrong behavior, visible defect |
| 🟡 **Medium** | Partially works, edge-case failures, dead code |
| 🔵 **Low** | Style, minor inefficiency, missing polish |

---

## 🔴 Critical (3)

### CR-01: `LayoutTree::close()` — always pops last node, ignores `_node_id`

**File:** `splatter-core/src/layout/mod.rs:234`  
**Symptom:** Closing any pane always closes the *last* leaf regardless of which one you clicked. The `_node_id` parameter is completely ignored (underscore-prefixed).  
**Root cause:** The method just does `self.nodes.pop()` without looking at the ID.  
**Fix:** Find the node by `node_id`, remove it, and promote its sibling.

### CR-02: `LayoutTree::get_node()` and `get_node_mut()` — always return `None`

**File:** `splatter-core/src/layout/mod.rs:283-286`  
**Symptom:** Any code that tries to find a node by ID gets `None`. This breaks split, close, focus, and any ID-based layout operations.  
**Root cause:** Body is `None` with no search logic.  
**Fix:** Search `self.nodes` for a matching ID, return `Some(node)`.

### CR-03: `LayoutTree::split()` on a Split node — discards the split node

**File:** `splatter-core/src/layout/mod.rs:163-222`  
**Symptom:** Splitting a split area (when the last node is a `Split`) creates two raw leaves from the split's children and discards the split node entirely. The tree structure is broken.  
**Root cause:** The `Split` arm extracts the split's children as leaves but never creates a new split node to replace the old one. The `self.nodes.push(split_node)` at line 227 only fires for the Leaf arm.  
**Fix:** After the `Split` arm, push the new `LayoutNode::Split` to `self.nodes` before returning.

---

## 🟠 High (7)

### H-01: `LayoutTree::focus_direction()` — no-op

**File:** `splatter-core/src/layout/mod.rs:239`  
**Symptom:** Arrow key / direction focus does nothing.  
**Fix:** Implement neighbor search across leaves, update focused node.

### H-02: `LayoutTree::focus_by_id()` — no-op

**File:** `splatter-core/src/layout/mod.rs:242`  
**Symptom:** Clicking a pane to focus it does nothing at the Rust level.  
**Fix:** Search for the node by ID and set as focused.

### H-03: Settings tabs — no active tab state, all sections render simultaneously

**File:** `web/src/components/Settings.tsx:49-60`  
**Symptom:** All five settings sections render at once, stacked on top of each other. No tab click handlers. User sees overlapping garbage.  
**Fix:** Add `activeTab` state, conditional rendering per tab, `onClick` handlers, and active tab styling.

### H-04: `useGhostty()` — re-init on every `cols`/`rows` change, never reinitializes

**File:** `web/src/hooks/useGhostty.ts:29`  
**Symptom:** `useEffect` deps are `[]`, so Ghostty only initializes once. If `cols`/`rows` change (pane resize), the effect doesn't re-run. But the resize is handled by the separate `resize()` callback — so this is mostly OK. However, if the container element changes (e.g., after a pane close), the terminal won't reattach.  
**Fix:** Either add `containerRef` to deps, or handle re-mount in the cleanup.

### H-05: Layout store — `root` never set on initial load when `get_layout` returns `null`

**File:** `web/src/components/Layout.tsx:28-35`  
**Symptom:** If `get_layout` returns `null` (first launch before any pane is created), the layout stays `null` and shows "No panes". But `App.tsx` creates a pane via `new_pane`, and the `layout-changed` listener should update `root`. The race between the initial fetch and the event is not handled.  
**Fix:** Add a `layoutInitialized` ref, and in `layout-changed` handler, set root even if initial fetch returned null.

### H-06: Tray Manager — `tick()` never called, tray status always `Idle`

**File:** `splatter-core/src/tray/mod.rs:87-103`  
**Symptom:** System tray always shows "Idle" regardless of agent count. The `tick()` method exists but is never called from `main.rs`.  
**Fix:** Call `tray.tick()` from the PTY drain loop with current agent states.

### H-07: `PluginHost::set_enabled()` — sets `state` to `Error("Disabled")` instead of a proper disabled state

**File:** `splatter-core/src/plugin/mod.rs:112-118`  
**Symptom:** Disabled plugin shows `state: Error("Disabled")` in the UI.  
**Fix:** Add `PluginState::Disabled` variant.

---

## 🟡 Medium (5)

### M-01: `spawn_agent()` function in `agent_commands.rs` is dead code

**File:** `splatter-core/src-tauri/src/agent_commands.rs:8-31`  
**Symptom:** Function declared but never registered in `tauri::generate_handler![]`.  
**Fix:** Remove or wire it up.

### M-02: `write_to_agent` — double-lock of `agents` mutex (wasteful, not harmful)

**File:** `splatter-core/src-tauri/src/agent_commands.rs:43-47`  
**Symptom:** Second lock creates shadow variable, immediately dropped. Waste of CPU cycles.  
**Fix:** Remove the second lock; the first `agents_guard` is already held.

### M-03: Ghostty type declarations (`ghostty-web.d.ts`) — duplicate `onResize`, wrong API

**File:** `web/src/types/ghostty-web.d.ts`  
**Symptom:** `onResize` declared twice. The type definitions are hand-written and don't match the actual `ghostty-web` package exports.  
**Fix:** Replace with `npm` package's `dist/index.d.ts` or remove hand-written types (the package provides its own types).

### M-04: `LayoutTree::split()` — Split node gets `id: 0` in JSON serialization

**File:** `splatter-core/src/layout/mod.rs:290`  
**Symptom:** Split nodes always have `id: 0` in the JSON sent to frontend. Frontend may expect unique IDs.  
**Fix:** Generate unique IDs for split nodes too.

### M-05: `LayoutTree::split()` — Split arm extracts `left.get_agent()` for right child

**File:** `splatter-core/src/layout/mod.rs:208`  
**Symptom:** When splitting a split node, the right child gets the left child's agent_id. This is a copy-paste bug — `right.get_agent()` was intended.  
**Fix:** Change `left.get_agent()` → `right.get_agent()`.

---

## 🔵 Low (3)

### L-01: Duplicate `listen("layout-changed")` registrations

**File:** `web/src/App.tsx`, `web/src/components/Layout.tsx`  
**Symptom:** Both files register a `layout-changed` listener. Each causes a `get_layout` fetch. Double IPC calls on every layout change.  
**Fix:** Keep it in one place (preferably App.tsx only).

### L-02: No Tauri icons in bundle config

**File:** `splatter-core/src-tauri/tauri.conf.json`  
**Symptom:** `"icon": []` — app has no tray/window icons.  
**Fix:** Add icon paths to the bundle config.

### L-03: `GhosttyTerminal.tsx` — listen cleanup returns a promise-wrapped dispose

**File:** `web/src/components/Ghostty/GhosttyTerminal.tsx:60-63`  
**Symptom:** `listen()` returns a promise. The cleanup function is `unsubPromise.then(unsub => unsub)`, which works but creates an unnecessary promise chain. If the component unmounts before the promise resolves, the dispose never runs.  
**Fix:** Use `.then()` properly or switch to the `once` pattern.

---

## File-by-File Summary

| File | Critical | High | Medium | Low |
|------|----------|------|--------|-----|
| `layout/mod.rs` | 3 | 2 | 2 | — |
| `agent/mod.rs` | — | — | — | — |
| `agent_commands.rs` | — | — | 2 | — |
| `layout_commands.rs` | — | — | — | — |
| `main.rs` | — | 1 | — | — |
| `Settings.tsx` | — | 1 | — | — |
| `useGhostty.ts` | — | 1 | — | — |
| `Layout.tsx` | — | 1 | — | — |
| `GhosttyTerminal.tsx` | — | — | — | 1 |
| `App.tsx` | — | — | — | 1 |
| `tray/mod.rs` | — | 1 | — | — |
| `plugin/mod.rs` | — | 1 | — | — |
| `ghostty-web.d.ts` | — | — | 1 | — |
| `tauri.conf.json` | — | — | — | 1 |
| **Total** | **3** | **7** | **5** | **3** |

---

## What Works

- ✅ Rust compiles (1 warning)
- ✅ TypeScript compiles (0 errors)
- ✅ 14 unit tests pass
- ✅ Vite build succeeds
- ✅ Ghostty WASM loads from `/ghostty-vt.wasm`
- ✅ `GhosttyTerminal` renders DOM elements
- ✅ Agent spawn → `new_pane` → `agent-spawned` event chain works (basic flow)
- ✅ PTY read loop drains output every 50ms
- ✅ Config load/save to TOML works
- ✅ Agent list sidebar renders
- ✅ Status bar shows working/done counts

## What Doesn't Work

- 🔴 Closing panes closes the wrong pane (CR-01)
- 🔴 Layout node lookup by ID always returns None (CR-02)
- 🔴 Splitting split nodes breaks tree structure (CR-03)
- 🟠 Settings tabs all render at once (H-03)
- 🟠 Arrow key / focus navigation does nothing (H-01, H-02)
- 🟠 System tray always shows idle (H-06)
- 🟠 Plugin disable shows as error state (H-07)
