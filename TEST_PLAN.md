# Splatter — Test Plan

## Testing Strategy

This document defines the complete testing strategy for Splatter, covering unit tests, integration tests, end-to-end tests, and platform-specific tests. Tests are organized by component and phase.

## Test Pyramid

```
        / E2E tests (few)
       / Integration tests (moderate)
      / Unit tests (many)
     /
```

## Component Test Matrix

### 1. Agent Launcher System

**What it does:** Spawns agent processes, manages PTY sessions, tracks agent lifecycle.

**Test types:** Unit, Integration

| Test | Type | Description | Priority |
|------|------|-------------|----------|
| `launch_pi_agent` | Unit | Spawn `pi` process in PTY, verify process exists and PTY attached | P0 |
| `launch_agent_with_env` | Unit | Spawn agent with custom env vars, verify env vars in process | P0 |
| `launch_agent_with_cwd` | Unit | Spawn agent with specific working directory, verify cwd in process | P0 |
| `launch_agent_failure` | Unit | Spawn non-existent binary, verify error handling | P1 |
| `launch_agent_timeout` | Unit | Agent fails to start within timeout, verify timeout handling | P1 |
| `agent_output_capture` | Integration | Write to PTY, verify output captured and forwarded | P0 |
| `agent_input_forward` | Integration | Send input to PTY, verify agent receives it | P0 |
| `agent_resize` | Integration | Resize PTY, verify agent receives new dimensions | P1 |
| `agent_signal_forward` | Integration | Send SIGINT/SIGTERM, verify agent receives it | P1 |
| `agent_exit_detection` | Integration | Agent exits, verify exit event emitted | P0 |
| `agent_crash_detection` | Integration | Agent crashes (SIGSEGV), verify crash event emitted | P1 |
| `multi_agent_concurrent` | Integration | Launch 5 agents concurrently, verify all run independently | P1 |
| `agent_env_interpolation` | Unit | `PID_TOKEN: ${env:PID_TOKEN}` resolves to actual env var | P1 |
| `agent_profile_load` | Unit | Load agent profile from YAML, verify all fields parsed | P2 |
| `agent_profile_validation` | Unit | Invalid agent profile rejected with clear error | P2 |
| `agent_profile_custom_fields` | Unit | Custom profile fields passed through to spawn | P2 |

**Test data:**

```yaml
# Test agent profiles
- name: "test-bash"
  command: "bash"
  args: ["-c", "echo hello && sleep 1"]
  expected_output: "hello"
- name: "test-fail"
  command: "nonexistent-command"
  expected_error: true
```

**Acceptance criteria:**

- All P0 tests pass 100%
- Agent process starts within 1 second
- PTY attached and I/O working
- Exit/crash events emitted within 500ms

---

### 2. Ghostty Web Terminal Rendering

**What it does:** Renders terminal output in WebView using ghostty-web.

**Test types:** Unit, Integration

| Test | Type | Description | Priority |
|------|------|-------------|----------|
| `init_ghostty_wasm` | Unit | Initialize ghostty-wasm, verify WASM loaded successfully | P0 |
| `open_terminal` | Unit | Create terminal instance, open into container, verify canvas created | P0 |
| `write_data` | Integration | Write VT data to terminal, verify rendered to canvas | P0 |
| `write_binary_data` | Integration | Write binary data (UTF-8 escape sequences), verify correct rendering | P0 |
| `write_large_output` | Integration | Write 100K lines of output, verify no crash/memory leak | P1 |
| `true_color` | Integration | Write ANSI escape with 24-bit color, verify correct color rendered | P1 |
| `unicode_complex` | Integration | Write Devanagari/Arabic text, verify proper rendering | P1 |
| `unicode_emoji` | Integration | Write emoji, verify proper rendering (wide character) | P1 |
| `mouse_tracking` | Integration | Enable mouse tracking, click terminal, verify events received | P1 |
| `bracketed_paste` | Integration | Enable bracketed paste, paste multi-line text, verify paste events | P1 |
| `osc52` | Integration | Terminal sends OSC 52 paste request, verify clipboard interaction | P2 |
| `scrollback_buffer` | Integration | Write 15K lines, scroll up, verify lines accessible | P1 |
| `resize_terminal` | Integration | Resize terminal (cols/rows), verify re-render | P1 |
| `resize_panic` | Integration | Resize from 80x24 to 5x5, verify no crash | P2 |
| `alternate_buffer` | Integration | Alternate screen buffer (vim mode), verify switch between normal/alternate | P2 |
| `input_latency` | Integration | Measure time from keypress to input event, verify < 16ms | P2 |
| `output_latency` | Integration | Measure time from write to render, verify < 16ms | P2 |
| `canvas_render` | Integration | Render 100 frames, verify no memory leak, no canvas allocation failure | P2 |
| `wasm_memory` | Integration | Run terminal for 5 minutes, verify no WASM memory leak | P2 |
| `ghostty_in_webkitgtk` | Integration | Verify ghostty-web runs in Tauri WebKitGTK on Linux | P0 |
| `ghostty_performance_baseline` | Benchmark | Render 50K lines, measure FPS and memory usage | P2 |

**Test data:**

```typescript
// VT escape sequence test data
const test_sequences = [
  // Colors
  "\x1b[38;2;255;100;50mRed-Green-Blue\x1b[0m",
  // Unicode
  "Devanagari: ऋषि",
  "Arabic: مرحبا",
  "Emoji: 🎉🔥✨",
  // Wide characters
  "Width: １２３",
  // Osc 52
  "\x1b]52;c;aGVsbG8=\x07",
  // Bracketed paste
  "\x1b[200~\x1b[201~",
];
```

**Acceptance criteria:**

- P0 tests pass 100%
- ghostty-web renders in WebKitGTK (Linux)
- No WASM memory leak over 5-minute test
- Input latency < 16ms (60fps)
- Output latency < 16ms (60fps)

---

### 3. Layout Engine (Pane Grid)

**What it does:** Manages BSP tree of panes, computes rectangles, handles splits/merges.

**Test types:** Unit, Integration

| Test | Type | Description | Priority |
|------|------|-------------|----------|
| `single_pane` | Unit | Create grid with single pane, verify rect covers 100% | P0 |
| `split_right` | Unit | Split pane right, verify two panes with correct widths | P0 |
| `split_down` | Unit | Split pane down, verify two panes with correct heights | P0 |
| `split_ratio_0.5` | Unit | Split with ratio 0.5, verify equal halves | P0 |
| `split_ratio_0.3` | Unit | Split with ratio 0.3, verify unequal halves | P1 |
| `close_pane_left` | Unit | Close left pane, verify right pane expands to full width | P0 |
| `close_pane_top` | Unit | Close top pane, verify bottom pane expands to full height | P1 |
| `close_last_pane` | Unit | Close only remaining pane, verify grid is empty | P1 |
| `close_reuse` | Unit | Close pane that will be reused (same ID), verify layout correct | P2 |
| `focus_next_right` | Unit | Focus pane, move right, verify next pane focused | P0 |
| `focus_next_down` | Unit | Focus pane, move down, verify next pane focused | P1 |
| `focus_cycle_wrap` | Unit | Focus cycle wraps around, verify circular navigation | P1 |
| `swap_panes` | Unit | Swap two panes, verify rectangles exchanged | P1 |
| `move_pane_to_parent` | Unit | Move pane from child A to child B of parent, verify layout | P2 |
| `zoom_pane` | Unit | Zoom pane, verify it covers full grid | P1 |
| `unzoom_pane` | Unit | Unzoom pane, verify original layout restored | P1 |
| `nested_splits` | Unit | Create complex nested layout (4+ splits), verify all rects correct | P2 |
| `resize_pane_delta` | Unit | Resize pane by delta, verify all adjacent panes adjust | P2 |
| `resize_pane_bounds` | Unit | Resize pane to minimum size, verify no negative dimensions | P2 |
| `preset_2panes_h` | Unit | Load "2-pane horizontal" preset, verify layout | P2 |
| `preset_2panes_v` | Unit | Load "2-pane vertical" preset, verify layout | P2 |
| `preset_3panes_row` | Unit | Load "3-pane row" preset, verify layout | P2 |
| `preset_2x2_grid` | Unit | Load "2×2 grid" preset, verify layout | P2 |
| `preset_custom` | Unit | Load custom preset from YAML, verify layout | P2 |
| `export_preset` | Unit | Export current layout as preset, verify YAML valid | P2 |
| `import_preset` | Unit | Import preset from YAML, verify layout | P2 |
| `layout_serialization` | Unit | Serialize layout to JSON, deserialize, verify same layout | P2 |

**Acceptance criteria:**

- All P0 tests pass 100%
- Pane rectangles never have negative dimensions
- Layout serialization/deserialization round-trips correctly
- Preset loading from YAML works

---

### 4. Agent Awareness Engine

**What it does:** Tracks agent state, builds activity history, manages agent lifecycle.

**Test types:** Unit, Integration

| Test | Type | Description | Priority |
|------|------|-------------|----------|
| `status_idle` | Unit | Agent starts, status set to idle | P0 |
| `status_working` | Unit | Agent output received, status set to working | P0 |
| `status_blocked` | Unit | Agent output blocked indicator, status set to blocked | P0 |
| `status_done` | Unit | Agent finishes, status set to done | P0 |
| `status_error` | Unit | Agent crashes, status set to error | P0 |
| `status_landing` | Unit | Agent starts, status set to launching, then idle | P1 |
| `status_timestamp` | Unit | Status transition recorded with timestamp | P0 |
| `status_last_transition` | Unit | last_transition_at updated on every status change | P1 |
| `activity_entry` | Unit | Activity entry created on status change | P1 |
| `activity_replay` | Integration | Replay activity log, verify all entries in order | P2 |
| `agent_duration` | Unit | Agent duration calculated from start to now | P1 |
| `output_bytes_counted` | Unit | Output bytes accumulated in stats | P2 |
| `output_lines_counted` | Unit | Output lines counted in stats | P2 |
| `agent_pin` | Unit | Pin agent, verify pin persisted | P1 |
| `agent_unpin` | Unit | Unpin agent, verify pin removed | P1 |
| `agent_group` | Unit | Add group to agent, verify group persisted | P2 |
| `agent_tags` | Unit | Add tags to agent, verify tags persisted | P2 |
| `agent_notes` | Unit | Create note for agent, verify note persisted | P2 |
| `agent_notes_list` | Unit | List notes for agent, verify all returned | P2 |
| `agent_resume_token` | Unit | Generate resume token on agent exit | P2 |
| `agent_handoff_context` | Unit | Capture handoff context from agent output | P2 |
| `agent_detection_type` | Unit | Detect agent type from process + pattern | P1 |
| `agent_detection_failed` | Unit | Agent not detected, status set to unknown | P1 |
| `agent_multiple_sessions` | Unit | Multiple agents with same profile, verify independent state | P2 |

**Acceptance criteria:**

- All P0 tests pass 100%
- Status transitions tracked with timestamps
- Activity log maintained in order
- Agent pin/group/tags persist across restarts

---

### 5. System Tray

**What it does:** Manages system tray icon, tooltip, menu, status color changes.

**Test types:** Integration, E2E

| Test | Type | Description | Priority |
|------|------|-------------|----------|
| `tray_create` | Integration | Create tray icon, verify icon visible in system tray | P0 |
| `tray_menu` | Integration | Right-click tray, verify menu items present | P0 |
| `tray_tooltip` | Integration | Set tooltip, verify tooltip shows on hover | P1 |
| `tray_icon_color` | Integration | Set colored icon (red/green/gray), verify icon color changes | P1 |
| `tray_click_show` | Integration | Left-click tray, verify window shown | P1 |
| `tray_right_click_menu` | Integration | Right-click tray, verify menu shown | P1 |
| `tray_quit` | Integration | Click "Quit" in tray menu, verify app exits | P1 |
| `tray_agent_count` | Integration | Set agent count in tray, verify count displayed | P2 |
| `tray_agent_status_summary` | Integration | Set status summary in tray tooltip, verify shown | P2 |
| `tray_no_agents` | Integration | No agents, verify tray still shows default icon | P1 |
| `tray_all_done` | Integration | All agents done, verify tray shows gray icon | P2 |
| `tray_all_working` | Integration | All agents working, verify tray shows green icon | P2 |
| `tray_mixed_status` | Integration | Mixed agent statuses, verify tray shows most critical | P2 |
| `tray_hide_on_minimize` | Integration | Minimize window, verify tray visible | P1 |
| `tray_show_on_restore` | Integration | Show from tray, verify window restored | P1 |
| `tray_linux` | Integration | Verify tray works on Linux (GTK/WebKitGTK) | P0 |
| `tray_appindicator` | Integration | Verify tray works with appindicator backend | P2 |

**Acceptance criteria:**

- P0 tests pass 100%
- Tray icon visible in system tray on Linux (GNOME/KDE/XFCE)
- Tray menu responds to clicks
- Tray icon color changes with agent status

---

### 6. Notification Engine

**What it does:** Dispatches native OS notifications with configurable triggers.

**Test types:** Unit, Integration

| Test | Type | Description | Priority |
|------|------|-------------|----------|
| `notify_send` | Integration | Send basic notification, verify visible on screen | P0 |
| `notify_agent_blocked` | Integration | Agent blocked, notification sent, verify visible | P0 |
| `notify_agent_done` | Integration | Agent done, notification sent, verify visible | P0 |
| `notify_sound_enabled` | Integration | Sound enabled, notification plays sound | P1 |
| `notify_sound_disabled` | Integration | Sound disabled, no sound played | P1 |
| `notify_focus_when_not_focused` | Integration | App not focused, notification shown | P0 |
| `notify_focus_when_focused` | Integration | App focused, notification suppressed | P1 |
| `notify_coalesce_window` | Integration | Multiple events in coalesce window, verify grouped | P1 |
| `notify_coalesce_off` | Integration | No coalesce, each event is separate notification | P1 |
| `notify_trigger_disabled` | Integration | Trigger disabled, no notification sent | P1 |
| `notify_trigger_condition` | Integration | Trigger with condition, verify condition evaluated | P2 |
| `notify_config_change` | Integration | Change notification config, verify new config used | P2 |
| `notify_dbus_linux` | Integration | Send via D-Bus on Linux, verify visible | P0 |
| `notify_send_cli` | Integration | Send via notify-send on Linux, verify visible | P1 |
| `notify_notification_center_macos` | Integration | Send via UserNotifications on macOS, verify in Notification Center | P0 |
| `notify_action_button` | Integration | Notification with action button, verify action triggered | P2 |
| `notify_persistent` | Integration | Persistent notification, verify stays until dismissed | P2 |
| `notify_expiry` | Integration | Notification with expiry, verify auto-dismiss | P2 |

**Acceptance criteria:**

- P0 tests pass 100%
- Notifications visible on Linux (GNOME/KDE/XFCE) via D-Bus
- Notifications visible on macOS via UserNotifications
- Notification triggers evaluated correctly

---

### 7. Global Hotkeys

**What it does:** Registers system-wide keyboard shortcuts.

**Test types:** Unit, Integration

| Test | Type | Description | Priority |
|------|------|-------------|----------|
| `hotkey_register_nav_prev` | Integration | Register "Cmd+Shift+Up" for prev pane, verify registered | P0 |
| `hotkey_register_nav_next` | Integration | Register "Cmd+Shift+Down" for next pane, verify registered | P0 |
| `hotkey_register_nav_cycle` | Integration | Register "Cmd+Tab" for cycle, verify registered | P0 |
| `hotkey_register_nav_focus` | Integration | Register "Cmd+H/J/K/L" for focus, verify registered | P0 |
| `hotkey_register_split` | Integration | Register "Cmd+Shift+V" for split, verify registered | P0 |
| `hotkey_register_zoom` | Integration | Register "Cmd+Z" for zoom, verify registered | P0 |
| `hotkey_register_agent` | Integration | Register "Cmd+I" for interrupt, verify registered | P0 |
| `hotkey_fire` | Integration | Press registered hotkey, verify handler called | P0 |
| `hotkey_fire_while_not_focused` | Integration | Press hotkey when app not focused, verify handler called | P0 |
| `hotkey_conflict` | Integration | Register hotkey already used by system, verify error handling | P1 |
| `hotkey_unregister` | Integration | Unregister hotkey, verify no longer fires | P1 |
| `hotkey_reregister` | Integration | Unregister then re-register same hotkey, verify works | P2 |
| `hotkey_config_change` | Integration | Change hotkey config, verify new hotkey registered | P2 |
| `hotkey_linux` | Integration | Verify hotkeys work on Linux (GTK/WebKitGTK) | P0 |
| `hotkey_global_scope` | Integration | Verify hotkeys registered system-wide (not just in window) | P1 |

**Acceptance criteria:**

- All P0 tests pass 100%
- Hotkeys fire even when app not focused
- Hotkey conflicts handled gracefully

---

### 8. Multi-Window Manager

**What it does:** Manages multiple windows, monitor detection, window state persistence.

**Test types:** Integration, E2E

| Test | Type | Description | Priority |
|------|------|-------------|----------|
| `window_create` | Integration | Create new window, verify window created | P0 |
| `window_label_unique` | Integration | Create window with duplicate label, verify error | P0 |
| `window_monitor_detect` | Integration | Detect all monitors, verify correct number | P0 |
| `window_on_monitor` | Integration | Create window on specific monitor, verify position | P1 |
| `window_auto_layout` | Integration | Auto-layout windows on multi-monitor, verify positions | P1 |
| `window_position_persist` | Integration | Close window, reopen, verify position restored | P1 |
| `window_size_persist` | Integration | Close window, reopen, verify size restored | P1 |
| `window_share_session` | Integration | Two windows share session, verify state synced | P2 |
| `window_independent_session` | Integration | Two windows independent sessions, verify isolated state | P2 |
| `window_monitor_change` | Integration | Remove monitor, verify window on removed monitor handled | P2 |
| `window_add_monitor` | Integration | Add new monitor, verify hotplug handled | P2 |
| `window_focus` | Integration | Switch focus between windows, verify correct focus | P2 |
| `window_zoom` | Integration | Zoom window to full screen, verify correct behavior | P2 |
| `window_minimize` | Integration | Minimize window, verify icon in taskbar/dock | P1 |
| `window_close_last` | Integration | Close last window, verify app exits (configurable) | P1 |
| `window_linux` | Integration | Verify window management works on Linux | P0 |

**Acceptance criteria:**

- P0 tests pass 100%
- Window created on correct monitor
- Window position/size restored on restart
- Monitor hotplug handled gracefully

---

### 9. Plugin Host

**What it does:** Loads, executes, and manages plugins with manifest and API.

**Test types:** Unit, Integration

| Test | Type | Description | Priority |
|------|------|-------------|----------|
| `plugin_load_manifest` | Unit | Load plugin manifest from YAML, verify parsed | P0 |
| `plugin_load_entry` | Unit | Load plugin JS entry point, verify loaded | P0 |
| `plugin_on_ready` | Unit | Plugin `onPluginReady` called on load, verify | P0 |
| `plugin_on_status_change` | Unit | Plugin `onAgentStatusChanged` called on status change, verify | P0 |
| `plugin_unload` | Unit | Unload plugin, verify `onPluginWillUnload` called | P1 |
| `plugin_error_handling` | Unit | Plugin throws error, verify error caught and logged | P1 |
| `plugin_permission_denied` | Unit | Plugin requests unallowed permission, verify denied | P1 |
| `plugin_http_fetch` | Unit | Plugin makes HTTP request, verify allowed with permission | P2 |
| `plugin_http_blocked` | Unit | Plugin makes HTTP request without permission, verify blocked | P2 |
| `plugin_marketplace_search` | Unit | Search marketplace for plugins, verify results returned | P2 |
| `plugin_marketplace_install` | Unit | Install plugin from marketplace, verify installed | P2 |
| `plugin_marketplace_update` | Unit | Update plugin from marketplace, verify updated | P2 |
| `plugin_marketplace_uninstall` | Unit | Uninstall plugin from marketplace, verify removed | P2 |
| `plugin_hot_reload` | Integration | Modify plugin JS file, verify hot-reload works | P2 |
| `plugin_multiple_concurrent` | Integration | Run 3 plugins concurrently, verify all work independently | P2 |
| `plugin_api_agent_list` | Unit | Plugin calls `splatter.agent.list()`, verify agent list returned | P2 |
| `plugin_api_settings_get` | Unit | Plugin calls `splatter.settings.get()`, verify config returned | P2 |
| `plugin_api_notification` | Unit | Plugin calls `splatter.notification.send()`, verify notification sent | P2 |
| `plugin_security_sandbox` | Unit | Plugin tries to access filesystem, verify sandboxed | P1 |
| `plugin_version_compat` | Unit | Plugin with incompatible version, verify rejected | P2 |

**Acceptance criteria:**

- P0 tests pass 100%
- Plugin manifest parsing works
- Plugin lifecycle hooks fire correctly
- Plugin sandbox prevents unauthorized access

---

### 10. Settings & Configuration

**What it does:** Loads/saves settings, migration, import/export.

**Test types:** Unit, Integration

| Test | Type | Description | Priority |
|------|------|-------------|----------|
| `config_load` | Unit | Load config.toml, verify all fields parsed | P0 |
| `config_default_values` | Unit | Missing config keys use defaults | P0 |
| `config_save` | Unit | Save config, verify written to disk | P0 |
| `config_reload` | Unit | Change config file, reload, verify new values | P1 |
| `config_migration_v1_v2` | Unit | Migrate from v1 to v2, verify values transferred | P1 |
| `config_migration_missing_v1` | Unit | No v1 config, migrate to v2, verify defaults used | P2 |
| `config_import` | Unit | Import config from file, verify loaded | P1 |
| `config_export` | Unit | Export config to file, verify valid TOML | P1 |
| `config_validate` | Unit | Invalid config rejected with error | P1 |
| `config_schema_strict` | Unit | Unknown fields rejected by schema | P2 |
| `config_per_agent_profile` | Unit | Load per-agent profile configs, verify merged | P2 |
| `config_watch_changes` | Unit | Watch config file for changes, verify reload on change | P2 |

**Acceptance criteria:**

- P0 tests pass 100%
- Config load/save round-trips correctly
- Config migration works for all version pairs
- Import/export produces valid TOML

---

### 11. Settings UI

**What it does:** In-app Settings panel with all categories.

**Test types:** E2E

| Test | Type | Description | Priority |
|------|------|-------------|----------|
| `settings_open` | E2E | Open Settings panel, verify UI visible | P0 |
| `settings_categories` | E2E | Verify all categories present (General, Terminal, Agents, Notifications, Hotkeys, Plugins, Crash) | P0 |
| `settings_change_terminal_font` | E2E | Change font, apply, verify font changed | P1 |
| `settings_change_font_size` | E2E | Change font size, apply, verify font size changed | P1 |
| `settings_change_scrollback` | E2E | Change scrollback lines, apply, verify scrollback changed | P1 |
| `settings_change_input_batch` | E2E | Change input batch delay, apply, verify batch delay changed | P2 |
| `settings_change_notification` | E2E | Toggle notification trigger, verify trigger state changed | P1 |
| `settings_change_hotkey` | E2E | Change hotkey binding, apply, verify new hotkey works | P2 |
| `settings_change_plugin` | E2E | Toggle plugin enable/disable, verify plugin state changed | P2 |
| `settings_change_crash_reporting` | E2E | Toggle crash reporting, verify setting changed | P2 |
| `settings_import_export` | E2E | Import/export settings via UI, verify works | P2 |
| `settings_validate_input` | E2E | Enter invalid value, verify validation error shown | P2 |

**Acceptance criteria:**

- P0 tests pass 100%
- All categories visible and functional
- Settings changes apply correctly

---

### 12. Auto-Update

**What it does:** Checks for updates, downloads, installs.

**Test types:** Integration

| Test | Type | Description | Priority |
|------|------|-------------|----------|
| `update_check_no_update` | Integration | Check for updates, no update available, verify no prompt | P0 |
| `update_check_update_available` | Integration | Check for updates, update available, verify prompt | P1 |
| `update_download` | Integration | Download update, verify progress shown | P1 |
| `update_install` | Integration | Install update, verify restart required | P1 |
| `update_skip` | Integration | Skip update, verify skip remembered | P2 |
| `update_error` | Integration | Update fails (network error), verify error shown | P2 |

**Acceptance criteria:**

- P0 tests pass 100%
- Update check works (even if no updates in test)

---

### 13. Crash Reporting

**What it does:** Captures Rust/JS crashes and sends to Sentry.

**Test types:** Unit, Integration

| Test | Type | Description | Priority |
|------|------|-------------|----------|
| `crash_rust` | Integration | Trigger Rust panic, verify crash captured | P0 |
| `crash_js` | Integration | Throw uncaught JS error, verify crash captured | P0 |
| `crash_enabled` | Integration | Crash reporting enabled, crash sent to Sentry | P1 |
| `crash_disabled` | Integration | Crash reporting disabled, crash NOT sent to Sentry | P1 |
| `crash_sentry_dsn` | Integration | Verify Sentry DSN configured correctly | P2 |

**Acceptance criteria:**

- P0 tests pass 100%
- Crashes captured and sent when enabled
- Crashes NOT sent when disabled

---

## E2E Test Scenarios

These are end-to-end tests that exercise the full product:

| Scenario | Description | Priority |
|----------|-------------|----------|
| `e2e_launch_agent` | Open Splatter → click "New Agent" → select Pi → see terminal → see agent status | P0 |
| `e2e_split_pane` | Open Splatter → click "New Agent" → click "Split Right" → click "New Agent" in new pane → see two agents | P0 |
| `e2e_hotkey_nav` | Open Splatter → open 2 panes → press `Cmd+Shift+Down` → verify focus moved to next pane | P0 |
| `e2e_hotkey_split` | Open Splatter → press `Cmd+Shift+V` → verify new pane created | P0 |
| `e2e_agent_done` | Launch agent that exits quickly → verify status changes to done → verify notification sent | P1 |
| `e2e_agent_blocked` | Launch agent that blocks → verify status changes to blocked → verify notification sent | P1 |
| `e2e_tray_status` | Launch agent → verify tray icon color changes → click tray → verify menu shows | P1 |
| `e2e_settings_change` | Open Settings → change font size → close → reopen → verify font size persisted | P1 |
| `e2e_layout_preset` | Open Splatter → click "Layout Presets" → select "2×2 grid" → verify 4 panes created | P2 |
| `e2e_plugin` | Open Settings → Plugins → install example plugin → verify plugin loaded | P2 |
| `e2e_multi_window` | Open Splatter → open 2 windows → verify both show same session → close one → verify other unaffected | P2 |
| `e2e_agent_resume` | Launch agent → exit agent → click "Resume" → verify agent restarted with same config | P2 |
| `e2e_agent_notes` | Launch agent → open notes → add note → close → reopen → verify note persisted | P2 |
| `e2e_agent_pin` | Launch agent → pin agent → close Splatter → reopen → verify agent still pinned | P2 |
| `e2e_agent_group` | Launch agent → add group "Work" → close Splatter → reopen → verify agent in "Work" group | P2 |
| `e2e_config_import_export` | Export config → modify exported file → import → verify settings changed | P2 |
| `e2e_large_output` | Launch agent → generate 100K lines of output → verify terminal doesn't freeze | P2 |
| `e2e_rapid_input` | Type rapidly for 30 seconds → verify no input loss | P2 |
| `e2e_stress_layout` | Create 20 panes → resize all → verify no crash | P2 |
| `e2e_stress_agents` | Launch 10 agents → verify all run correctly → terminate all → verify all exit cleanly | P2 |

---

## Test Execution

### Unit Tests

```bash
# Rust unit tests
cargo test --lib

# TypeScript unit tests
npm test -- --unit
```

### Integration Tests

```bash
# Rust integration tests
cargo test --test integration

# Integration tests (TypeScript)
npm test -- --integration
```

### E2E Tests

```bash
# E2E tests with Playwright
npm test -- --e2e
```

### All Tests

```bash
npm run test:all
```

---

## Test Environments

| Environment | Purpose | Setup |
|-------------|---------|-------|
| **Unit** | Fast, no platform deps | Pure Rust + JS |
| **Integration** | Platform-specific | Linux (GTK/WebKitGTK), macOS (AppKit), Windows (WebView2) |
| **E2E** | Full product | Linux (GTK/WebKitGTK), macOS (AppKit), Windows (WebView2) |

---

## CI/CD Test Integration

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
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

      - name: Install dependencies
        run: |
          npm ci
          cargo build --lib

      - name: Run unit tests
        run: npm test -- --unit

      - name: Run integration tests
        run: npm test -- --integration

      - name: Run E2E tests
        run: npm test -- --e2e
```

---

## Performance Benchmarks

| Metric | Target | Test |
|--------|--------|------|
| Input latency | < 16ms | Type rapidly, measure time to input event |
| Output latency | < 16ms | Write data, measure time to render |
| Layout ops | < 50ms | Split/merge/resize, measure time |
| Window create | < 500ms | Create window, measure time to visible |
| Agent spawn | < 1000ms | Spawn agent, measure time to PTY attached |
| WASM init | < 200ms | Initialize ghostty-wasm, measure time |
| Config load | < 50ms | Load config.toml, measure time |
| Plugin load | < 200ms | Load plugin, measure time |
| Memory (idle) | < 200 MB | App idle, measure RSS |
| Memory (5 agents) | < 500 MB | 5 agents running, measure RSS |
| Memory (10 agents) |
