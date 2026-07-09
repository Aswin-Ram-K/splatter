# Splatter — Build, Audit, Fix & Verify Plan

**Last updated:** 2026-07-09  
**Status:** Phase 0 ✅ COMPLETE — Phase 1 ✅ COMPLETE

---

## Overview

This plan covers a full codebase audit, all bug fixes, and a comprehensive rebuild.  
Structure: **Audit → Fix → Build → Verify** for each phase.

**18 bugs found** across all layers: 3 🔴 Critical, 7 🟠 High, 5 🟡 Medium, 3 🔵 Low.

---

## Phase 0 — Layout Engine Fix (Critical Bugs)

**Goal:** Fix the 3 critical layout bugs that break pane operations.

### 0.1 ✅ `close()` — FIXED

**File:** `splatter-core/src/layout/mod.rs`  
**Bug:** Always popped last node, ignored `_node_id`.  
**Fix:** Implemented `remove_leaf_recursive()` that traverses the BSP tree, finds the leaf by ID, and promotes its sibling (removing the Split parent when one leaf is gone).

**Tests added:** `test_close_by_id`, `test_close_single_pane_fails`, `test_close_removes_and_promotes`


```rust
// CURRENT (broken):
pub fn close(&mut self, _node_id: NodeId) -> bool {
    if self.nodes.len() > 1 {
        self.nodes.pop();
        true
    } else {
        false
    }
}
```

```rust
// FIXED:
pub fn close(&mut self, node_id: NodeId) -> bool {
    if self.nodes.len() <= 1 {
        return false;
    }

    // Find the node to close
    let node_index = self.nodes.iter().position(|n| {
        if let LayoutNode::Leaf { id, .. } = n {
            *id == node_id
        } else {
            false
        }
    });

    if let Some(idx) = node_index {
        // Check if parent exists (for sibling promotion)
        if let Some(parent) = self.nodes.get(idx.saturating_sub(1)) {
            if parent.is_split() {
                // Promote sibling
                if let LayoutNode::Split { direction, ratio, left, right } = parent {
                    let sibling = if left.id() == &node_id { right } else { left };
                    *parent = *sibling;
                }
            }
        }
        self.nodes.remove(idx);
        true
    } else {
        false
    }
}
```

**Verify:**

- [ ] `cargo test --package splatter-core --lib` still passes
- [ ] Close last pane → window closes (single pane → nothing)
- [ ] Close one pane in a split → other pane fills the space

### 0.2 ✅ `get_node()` / `get_node_mut()` — FIXED

**File:** `splatter-core/src/layout/mod.rs`  
**Bug:** Stubs always returned `None`.  
**Fix:** Implemented recursive `find_node_recursive()` / `find_node_mut_recursive()` that traverse the BSP tree by matching leaf IDs.

**Tests added:** `test_get_node`


```rust
// FIXED:
pub fn get_node(&self, id: NodeId) -> Option<&LayoutNode> {
    self.nodes.iter().find(|n| {
        if let LayoutNode::Leaf { id: node_id, .. } = n {
            *node_id == id
        } else {
            false
        }
    })
}

pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut LayoutNode> {
    self.nodes.iter_mut().find(|n| {
        if let LayoutNode::Leaf { id: node_id, .. } = n {
            *node_id == id
        } else {
            false
        }
    })
}
```

**Verify:**

- [ ] `cargo test --package splatter-core --lib` still passes
- [ ] Add a test: create tree → split → `get_node(new_id)` returns Some

### 0.3 — Fix `split()` on Split node

**File:** `splatter-core/src/layout/mod.rs:227`  
**Bug:** Split arm doesn't push the new split node to `self.nodes`.

```rust
// CURRENT (broken — ends without push):
Some(LayoutNode::Split { ... }) => {
    ...
    (left_child, right_child, new_id)  // returns but never pushes split_node!
}
```

```rust
// FIXED — add push before split arm returns:
Some(LayoutNode::Split { direction: _, ratio: _, left, right }) => {
    let new_id = self.next_id;
    self.next_id += 1;
    let current_rect = left.leaf_rect().unwrap_or_else(Rect::full_screen);
    let (left_rect, right_rect) = match direction {
        // ... same split logic ...
    };
    let left_child = LayoutNode::Leaf { ... };
    let right_child = LayoutNode::Leaf { ... };
    
    let split_node = LayoutNode::Split {
        direction,
        ratio,
        left: Box::new(left_child),
        right: Box::new(right_child),
    };
    self.nodes.push(split_node);  // ← ADD THIS
    
    new_id
}
```

**Fix also:** Copy-paste bug on line 208 — `left.get_agent()` → `right.get_agent()`

**Verify:**

- [ ] Split a split pane → expect 3 panes (not 2)
- [ ] Right child gets correct agent_id (not left's)

### 0.4 ✅ Additional fixes — COMPLETE

- `leaf_count()` / `leaf_ids()` — now traverse tree recursively
- `new_leaf()` — now splits the root instead of appending to flat list
- `get_pane_size()` / `set_pane_agent()` / `leaves()` — now traverse tree
- `layout_commands.rs` — updated caller for new `set_pane_agent(&str)` signature
- Full build verified: `cargo build --release` ✓

### Test Results

```
running 20 tests
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All 14 original tests + 6 new tests pass.


```bash
cd splatter-core/src-tauri && cargo tauri dev
```

**Verify:**

- [ ] No schema warnings

---

## Phase 1 — High-Bug Fixes

### 1.1 — Implement `focus_direction()` and `focus_by_id()`

**File:** `splatter-core/src/layout/mod.rs:239-242`  
**Bug:** Empty implementations — arrow keys do nothing.

```rust
pub fn focus_by_id(&mut self, node_id: NodeId) {
    if self.nodes.iter().any(|n| {
        if let LayoutNode::Leaf { id, .. } = n {
            *id == node_id
        } else {
            false
        }
    }) {
        self.nodes.retain(|n| {
            if let LayoutNode::Leaf { id, .. } = n {
                *id == node_id
            } else {
                true  // keep splits
            }
        });
    }
}

pub fn focus_direction(&mut self, direction: FocusDirection) {
    let leaf_ids: Vec<NodeId> = self.leaf_ids();
    if leaf_ids.is_empty() { return; }
    
    let focused = self.focused_id();
    let idx = match focused {
        Some(id) => leaf_ids.iter().position(|&i| i == id).unwrap_or(0),
        None => 0,
    };
    
    let next_idx = match direction {
        FocusDirection::Next => (idx + 1) % leaf_ids.len(),
        FocusDirection::Previous => {
            if idx == 0 { leaf_ids.len() - 1 } else { idx - 1 }
        }
        // Left/Right/Up/Down can map to next/prev for now
        FocusDirection::Right | FocusDirection::Down | FocusDirection::Next => (idx + 1) % leaf_ids.len(),
        FocusDirection::Left | FocusDirection::Up | FocusDirection::Previous => {
            if idx == 0 { leaf_ids.len() - 1 } else { idx - 1 }
        }
    };
    
    self.focus_by_id(leaf_ids[next_idx]);
}
```

**Verify:**

- [ ] Arrow keys cycle through panes
- [ ] Ctrl+h/j/k/l focus adjacent panes

### 1.2 — Fix Settings tabs

**File:** `web/src/components/Settings.tsx`  
**Bug:** All 5 sections render stacked, no tab state, no click handlers.

Fix:

- [ ] Add `activeTab` state to `Settings` component
- [ ] Add `onClick` handlers to tab buttons
- [ ] Add `activeTab === tab` conditional for tab styling (border-b-blue-500)
- [ ] Wrap tab content in `if (activeTab === tab) { ... }` conditional
- [ ] Default to `'terminal'` tab on open

### 1.3 — Wire Tray Manager tick()

**File:** `splatter-core/src-tauri/src/main.rs`  
**Bug:** `tick()` never called, tray always idle.

In the PTY drain loop in `main.rs`, add tray update:

```rust
// In the drain loop, after draining outputs:
{
    let tray_state = app.state::<std::sync::Arc<std::sync::Mutex<TrayManager>>>();
    let mut tray_guard = tray_state.lock().unwrap();
    
    let agent_states: Vec<tray::AgentStateSummary> = agents_guard.iter()
        .map(|(id, session)| tray::AgentStateSummary {
            status: session.status,
        })
        .collect();
    
    tray_guard.tick(&agent_states);
}
```

**Verify:**

- [ ] Launch with an agent → tray shows "Active" with green dot
- [ ] Agent finishes → tray updates counts

### 1.4 — Fix PluginHost::set_enabled()

**File:** `splatter-core/src/plugin/mod.rs:112-118`  
**Bug:** Sets state to `Error("Disabled")` instead of proper disabled state.

Add `PluginState::Disabled` variant and use it:

```rust
pub enum PluginState {
    Loading,
    Ready,
    Disabled,
    Error(String),
}
```

**Verify:**

- [ ] Disable plugin → UI shows "Disabled" (not "Error")

---

## Phase 1 ✅ — High-Bug Fixes COMPLETE

### 1.1 ✅ `focus_direction()` / `focus_by_id()` — FIXED

**File:** `splatter-core/src/layout/mod.rs`  
**Bug:** Empty implementations — arrow keys did nothing.  
**Fix:** Added `focused_id: Option<NodeId>` field to `LayoutTree`. Implemented cycle-through navigation — all directional keys map to prev/next leaf in the leaf list.

**Tests:** `test_focus_by_id`, `test_focus_by_id_invalid`, `test_focus_direction`, `test_focus_cycles_all_leaves`

### 1.2 ✅ Settings Tabs — FIXED

**File:** `web/src/components/Settings.tsx`  
**Bug:** All 5 sections stacked, no tab state, no click handlers.  
**Fix:** Added `useState("terminal")` for active tab, `onClick` handlers on tab buttons, `activeTab === tab` conditional rendering per section, blue border on active tab.

### 1.3 ✅ Tray Manager `tick()` — FIXED

**File:** `splatter-core/src-tauri/src/main.rs`  
**Bug:** `tick()` never called, tray always idle.  
**Fix:** Added tray state clone in PTY drain loop. After draining agent outputs, aggregates agent statuses and calls `tray.tick()`. Also added `AgentManager.iter()` method.

### 1.4 ✅ PluginHost `set_enabled()` — FIXED

**File:** `splatter-core/src/plugin/mod.rs`  
**Bug:** Used `PluginState::Error("Disabled")` for disabled plugins.  
**Fix:** Added `PluginState::Disabled` variant, now uses proper state.

---

## Phase 2 — Medium Bug Fixes

### 2.1 — Remove dead code

- [ ] Remove `spawn_agent()` from `agent_commands.rs` (unused)
- [ ] Remove unused `HotkeyConfig` helper methods in `hotkey/mod.rs`
- [ ] Remove duplicate `onResize` from `ghostty-web.d.ts`
- [ ] Add `#[allow(dead_code)]` or wire up remaining warnings

### 2.2 — Fix `write_to_agent` double-lock

**File:** `splatter-core/src-tauri/src/agent_commands.rs:43-47`  
Remove the second `agents.lock()` — the first `agents_guard` is still in scope.

### 2.3 — Fix Ghostty type declarations

**File:** `web/src/types/ghostty-web.d.ts`  
Replace with proper types from `ghostty-web` package. Check if the npm package already provides types (`"types": "./dist/index.d.ts"` in package.json).

If so, remove the hand-written file and import from `ghostty-web` directly.

### 2.4 — Fix split node ID in JSON serialization

**File:** `splatter-core/src/layout/mod.rs:290`  
Split nodes should have unique IDs, not hardcoded `0`.

```rust
// In json_serialize_node for Split:
serde_json::json!({
    "id": node_id_for_split,  // generate unique
    ...
})
```

### 2.5 — Fix Layout.tsx duplicate listeners

**File:** `web/src/components/Layout.tsx`  
Remove the `listen("layout-changed")` handler since App.tsx already handles it.

---

## Phase 3 — Runtime & UX Stabilization

### 3.1 — Verify Ghostty WASM loads in production build

- [ ] Build release: `cargo build --release`
- [ ] Launch: `./target/release/splatter`
- [ ] Check browser console for `[Ghostty] Init failed` messages
- [ ] Verify terminal renders with xterm characters

### 3.2 — Verify agent spawn flow end-to-end

- [ ] Launch app → expect single pane
- [ ] Check `agent-spawned` event fires with correct `agent_id` and `layout_node_id`
- [ ] Verify agent appears in sidebar
- [ ] Type in terminal → expect PTY echo

### 3.3 — Verify PTY output works

- [ ] Agent spawns → expect shell prompt in terminal
- [ ] Type commands → expect output in terminal
- [ ] Resize window → expect terminal to resize

### 3.4 — Verify layout operations

- [ ] Split pane → expect 2 panes
- [ ] Close pane → expect remaining pane to fill space
- [ ] Arrow keys → expect pane focus to cycle

---

## Phase 4 — Polish & Packaging

### 4.1 — Add app icons

**File:** `splatter-core/src-tauri/tauri.conf.json`

- [ ] Add tray icon (16x16, 32x32, 64x64)
- [ ] Add window icon (256x256)
- [ ] Verify `tauri.conf.json` `"icon": ["../../resources/icons/..."]`

### 4.2 — Build installers

```bash
cd splatter-core/src-tauri
cargo tauri build
```

- [ ] `.deb` builds successfully
- [ ] `.AppImage` builds successfully
- [ ] Both install and run correctly

### 4.3 — Create `.desktop` file

- [ ] `~/.local/share/applications/splatter.desktop`
- [ ] Icon path set correctly
- [ ] MimeType, Categories, StartupWMClass filled in

### 4.4 — Final cleanup

- [ ] Remove all `console.log` debug statements
- [ ] Remove loading indicator from index.html
- [ ] Commit all changes
- [ ] Push to origin/main

---

## Verification Checklist

### Build

- [ ] `cargo check` — zero warnings
- [ ] `cargo test --package splatter-core --lib` — 14+ tests pass
- [ ] `npx tsc` — zero errors
- [ ] `npm run build` — zero errors
- [ ] `cargo build --release` — successful

### Runtime

- [ ] App launches (PID check)
- [ ] UI renders (not blank)
- [ ] Sidebar shows "AgentList"
- [ ] Terminal renders Ghostty (not empty)
- [ ] Status bar shows counts
- [ ] Settings opens with working tabs
- [ ] Arrow keys cycle panes
- [ ] Split works → 2 panes
- [ ] Close works → remaining pane fills
- [ ] No console errors
- [ ] No connection to localhost:5173
- [ ] No connection to localhost:15173

### Packaging

- [ ] `.deb` installs
- [ ] `.AppImage` runs
- [ ] Desktop shortcut works
- [ ] Icons visible in tray/menu

---

## Known Issues (Pre-existing)

| Issue | Impact | Phase |
|-------|--------|-------|
| Ghostty WASM chunk >500KB (637KB) | Bundle size warning | Phase 4 |
| `notify-send` used for Linux notifications | Requires X11/Wayland session | Phase 4 |
| Plugin system only logs, no JS execution | Feature stub | Out of scope |
| Crash reporting disabled by default | No crash data | Out of scope |
| Tray icon not defined in config | No icons | Phase 4 |
| Hotkey registry exists but unused | Feature stub | Out of scope |
| Window state manager exists but unused | Feature stub | Out of scope |

---

## Git Commands

```bash
# After each phase:
git add -A
git commit -m "Phase X.Y: <description>"

# Push:
git push origin main
```
