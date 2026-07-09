# Splatter — Multi-Phase Build/Audit/Fix/Verify Plan

**Created:** 2026-07-09
**Based on:** Full project audit (18 bugs found)
**Goal:** Ship a working Splatter binary that launches, loads Ghostty WASM, renders panes, and accepts keyboard input

---

## Phase 0: Environment Baseline (30 min)

**Objective:** Establish a clean, verifiable starting point

| Step | Action | Verification |
|------|--------|--------------|
| 0.1 | `cargo clean && cargo build --release` | Clean build succeeds |
| 0.2 | `cd web && npm run build` | Frontend builds, WASM copied to `dist/` |
| 0.3 | `cargo test --package splatter-core --lib` | All 14 tests pass |
| 0.4 | Snapshot current state: `git add -A && git commit -m "PRE-AUDIT-BASELINE"` | Git baseline committed |
| 0.5 | Launch app on desktop: `WAYLAND_DISPLAY=wayland-0 ./target/release/splatter` | App launches, note all errors |

**Go/No-Go:** Phase 0 must complete before proceeding.

---

## Phase 1: Layout Bridge — Fix BSP Tree ↔ Frontend Sync (Day 1 AM, ~3h)

**Bugs Addressed:** C2 (Flat array vs tree), H1 (Store sync), H3 (Preset not applied), M5 (Node IDs)

### 1.1 Backend: `get_layout()` Returns Proper BSP Tree

**Files:** `layout_commands.rs`

```rust
// BEFORE: returns flat array of leaves
serde_json::to_value(pane_data)

// AFTER: returns full tree with split/leaf structure
// Build recursive tree from LayoutTree's node structure
fn to_json_tree(node: &LayoutNode) -> serde_json::Value { ... }
```

**Changes:**

- Add `to_tree_json(&self, node: &LayoutNode) -> serde_json::Value` on `LayoutTree`
- `get_layout()` returns `{"root": <tree>}` or `null`
- Each node: `{ "id": N, "type": "split"|"leaf", "direction": "...", "ratio": 0.5, "left": {...}, "right": {...}, "rect": {...}, "agent_id": "..." }`

### 1.2 Backend: Fix `set_preset()`

**Files:** `layout_commands.rs`

```rust
// BEFORE: creates local preset, drops it
let _ = preset;

// AFTER: sets the layout
let mut guard = layout.lock().map_err(...)?;
*guard = preset;
```

### 1.3 Frontend: Layout.tsx — Parse Tree Structure

**Files:** `Layout.tsx`, `layoutStore.ts`

```ts
// BEFORE: setRoot(layout) stores flat array as-is
// AFTER: setRoot({ root: layout.root, panes: extractPanes(layout.root) })
```

- Parse `{ root: { ... } }` into tree structure
- Extract panes from leaf nodes into `panes` map
- `renderNode()` works correctly with tree structure

### 1.4 Frontend: Use Counter Instead of Date.now()

**Files:** `layoutStore.ts`

```ts
let nodeIdCounter = 1000;
const nextId = () => nodeIdCounter++;
```

**Tests:**

```bash
cargo test --package splatter-core --lib  # 14 tests pass
cd web && npx tsc --noEmit  # zero errors
cd web && npm run build  # builds
```

**Verification:** Launch app → should see one pane with agent, sidebar shows agent

---

## Phase 2: Ghostty WASM Loading Fix (Day 1 PM, ~2h)

**Bugs Addressed:** C1 (WASM "Connection refused"), M3 (onResize double-fire)

### 2.1 WASM Path Resolution

**Files:** `useGhostty.ts`

```ts
// BEFORE: init() tries multiple paths
await init();

// AFTER: Explicit path for Tauri environments
const wasmPath = import.meta.env.TAURI_PLATFORM === 'linux'
  ? '/ghostty-vt.wasm'  // Tauri serves from dist/ root
  : undefined;
await init(wasmPath);
```

**Alternative approach (more robust):**
Add a `<script src="/ghostty-vt.wasm" type="wasm">` to `index.html` so the WASM is preloaded and the module can find it via `import.meta.url`.

### 2.2 Tauri CSP Update

**Files:** `tauri.conf.json`

```json
"security": {
  "csp": "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: https://asset.localhost; connect-src 'self' asset: https://asset.localhost tauri://localhost http://localhost:5173;"
}
```

Add `connect-src` to allow fetch to `/ghostty-vt.wasm`.

### 2.3 Fix `onResize` Double-Fire

**Files:** `useGhostty.ts`

```ts
// Add initial call guard
let initialResizeSent = false;
term.onResize((resize) => {
  if (initialResizeSent && onResize) {
    onResize(resize.cols, resize.rows);
  }
});
// Send initial resize explicitly once
if (onResize && !initialResizeSent) {
  initialResizeSent = true;
  onResize(term.cols, term.rows);
}
```

### 2.4 Vite Plugin: Ensure WASM in Production

**Files:** `vite.config.ts`

Already handles `closeBundle` → copy to `dist/`. Verify no changes needed.

**Tests:**

```bash
cargo build --release  # succeeds
# Launch on desktop → no "Could not connect to localhost" error
# Ghostty terminal renders properly
```

**Verification:** App launches, terminal renders with cursor, no WASM errors in console

---

## Phase 3: Ghostty Input/Output Fix (Day 2 AM, ~2.5h)

**Bugs Addressed:** C3 (writeInput broken), C4 (onData naming), H4 (Focus)

### 3.1 Fix `writeInput` to Use Ghostty's Input API

**Files:** `useGhostty.ts`

```ts
// BEFORE: term.write(data) — writes to display
// AFTER: trigger input via Ghostty's input mechanism
const writeInput = useCallback((data: Uint8Array) => {
  if (termRef.current) {
    // Ghostty doesn't expose direct input API, but the onData emitter
    // IS the input channel. We need to write directly to the WASM terminal's input.
    // Use input() method instead of write()
    termRef.current.input(new TextDecoder().decode(data), true);
  }
}, []);
```

**Actually:** The Ghostty Terminal class has `input(data: string, wasUserInput: boolean)`. Since `data` is `Uint8Array`, we need to convert:

```ts
const writeInput = useCallback((data: Uint8Array) => {
  if (termRef.current) {
    const text = new TextDecoder().decode(data);
    termRef.current.input(text, true);  // true = trigger onData
  }
}, []);
```

### 3.2 Fix Focus Behavior

**Files:** `GhosttyTerminal.tsx`

```tsx
// BEFORE: container div gets focus
<div ref={containerRef} tabIndex={0} ...>

// AFTER: let Ghostty's internal textarea handle focus
// Ghostty.open() creates its own contenteditable+textarea.
// We just need to pass focus through:
const handleFocus = useCallback(() => {
  // Ghostty's open() creates textarea - focus it
  // The textarea is a child of the container
  if (containerRef.current) {
    const textarea = containerRef.current.querySelector('textarea');
    if (textarea) (textarea as HTMLTextAreaElement).focus();
  }
}, []);
```

### 3.3 Rename `onOutput` → `onInput` in useGhostty

**Files:** `useGhostty.ts`

```ts
// Renaming for clarity:
interface UseGhosttyOptions {
  onInput?: (data: Uint8Array) => void;  // Keyboard input FROM terminal → TO PTY
  onResize?: (cols: number, rows: number) => void;
}
```

Update `GhosttyTerminal.tsx` to use `onInput`.

### 3.4 Fix Terminal Resize Calculation

**Files:** `GhosttyTerminal.tsx`

```ts
// BEFORE: fixed pixel ratios
cols: Math.max(10, Math.floor(rect.width / 8)),
rows: Math.max(3, Math.floor(rect.height / 16)),

// AFTER: use Ghostty's actual cell dimensions
// Ghostty sets charWidth/charHeight on the renderer after open()
// We'll use a more accurate estimate based on font size
const charWidth = settings?.terminal?.font_size || 15;
const charHeight = charWidth * 1.5;  // Typical monospace ratio
cols: Math.max(10, Math.floor(rect.width / (charWidth * 0.6)));
rows: Math.max(3, Math.floor(rect.height / charHeight));
```

**Tests:**

```bash
cargo test --package splatter-core --lib  # 14 tests pass
cd web && npx tsc --noEmit  # zero errors
```

**Verification:**

- Type in terminal → PTY receives input → shell responds
- Terminal output → appears in display
- Focus switching between panes works

---

## Phase 4: Layout Store Cleanup (Day 2 PM, ~2h)

**Bugs Addressed:** H2 (Duplicate IDs), H5 (Race condition), M1 (Settings), M2 (Copy-paste), M6 (Dead code)

### 4.1 Remove Frontend `splitPane()` from Store

**Files:** `layoutStore.ts`

The frontend store's `splitPane()` method creates local state that's never synced with the backend. Remove it entirely. The only way to split is via Tauri `split_pane` command (triggered from keyboard shortcuts).

```ts
// REMOVE: splitPane() method from LayoutStore
// REMOVE: setPreset() method (use Tauri command instead)
// KEEP: splitPane that calls Tauri API
splitPane: async (direction: "vertical" | "horizontal") => {
  const id = get().focusedNodeId;
  if (!id) return;
  const newId = await invoke<number>("split_pane", {
    direction,
    ratio: 0.5,
  });
  return newId;
},
```

### 4.2 Fix Agent Spawn Race Condition

**Files:** `App.tsx`

```ts
// Use a ref to prevent double-mount spawning
const spawnedRef = useRef(false);
useEffect(() => {
  if (spawnedRef.current) return;
  spawnedRef.current = true;
  // ... spawn logic
}, []);  // empty deps — only runs once
```

### 4.3 Fix Settings UI

**Files:** `Settings.tsx`

- Wrap Hotkeys section in `<SettingsSection>`
- Remove all commented-out `<SettingsSection title="Plugins">` blocks
- Keep single `<PluginList />` inside Plugins section

### 4.4 Remove Dead Code

**Files:** `main.rs`, `agent_commands.rs`

```rust
// REMOVE from invoke_handler: agent_commands::spawn_agent
// KEEP: All other commands
```

**Tests:**

```bash
cargo test --package splatter-core --lib  # 14 tests pass
cd web && npm run build  # builds
```

**Verification:**

- Split pane creates two working terminals
- Close pane removes one terminal
- Settings UI renders correctly
- No dead code warnings

---

## Phase 5: Release Build & Desktop Verification (Day 3, ~1h)

### 5.1 Final Release Build

```bash
cd /home/zer0null/projects/herdr-web-dash/splatter
cargo build --release
cargo clippy --package splatter-core  # zero warnings (or documented)
cd web && npm run build
```

### 5.2 Desktop Integration

```bash
# Verify desktop shortcut
ls -la ~/.local/share/applications/splatter.desktop

# Launch on display
WAYLAND_DISPLAY=wayland-0 DISPLAY=:0 ./target/release/splatter &

# Verify process
ps aux | grep splatter
```

### 5.3 E2E Functional Tests

| Test | Expected |
|------|----------|
| App launches | No WASM errors, terminal renders |
| Single pane | Agent running, shell prompt visible |
| Type `ls` | Output appears in terminal |
| Split pane (Ctrl+Shift+E) | Two panes appear |
| Type in second pane | Shell prompt, accepts input |
| Close pane (Ctrl+D) | Returns to single pane |
| Settings opens | Modal renders, tabs work |
| Agent list sidebar | Shows running agents |
| Status bar | Counts correct |

### 5.4 Git Commit

```bash
git add -A
git commit -m "fix: Layout bridge, WASM loading, Ghostty input/output fixes

- Fix get_layout() to return BSP tree structure
- Fix WASM path resolution for Tauri production
- Fix Ghostty writeInput → term.input() for PTY writes
- Remove frontend splitPane() (use Tauri API)
- Fix settings UI (hotkeys section, dead code removal)
- Fix agent spawn race condition

Addresses: C1-C4, H1-H5, M1-M6"
```

---

## 📋 Summary Table

| Phase | Duration | Bugs Fixed | Risk Level |
|-------|----------|------------|------------|
| 0: Environment Baseline | 30 min | — | None |
| 1: Layout Bridge | 3h | C2, H1, H3, M5 | Low |
| 2: WASM Loading | 2h | C1, M3 | Medium |
| 3: Input/Output | 2.5h | C3, C4, H4, M4 | Medium |
| 4: Store Cleanup | 2h | H2, H5, M1, M2, M6 | Low |
| 5: Build & Verify | 1h | L1-L3 (cleanup) | Low |
| **Total** | **~11h** | **17/18** | **Low-Medium** |

**Remaining:** L1-L3 (unused code removal, Clippy warnings) — can be done as cleanup after Phase 5.

---

## 🚦 Go/No-Go Checkpoints

| Checkpoint | Condition | Action if Fail |
|------------|-----------|----------------|
| Pre-Phase 1 | Clean build, 14 tests | Fix build before proceeding |
| Post-Phase 1 | `get_layout()` returns tree, layout renders | Redesign tree serialization |
| Post-Phase 2 | WASM loads, no console errors | Investigate Tauri CSP/asset serving |
| Post-Phase 3 | Keyboard input reaches PTY, output displays | Fix Ghostty input handler |
| Post-Phase 4 | Split/close panes works from UI | Debug Tauri ↔ frontend sync |
| Post-Phase 5 | App launches on desktop, terminal works | Ship as is, fix remaining later |
