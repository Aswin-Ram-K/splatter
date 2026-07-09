/**
 * Settings store — Zustand state for app settings.
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, TrayStatus } from '@/types';

interface SettingsStore {
  settings: AppSettings;
  trayStatus: TrayStatus;
  notificationsEnabled: boolean;

  updateSettings: (updates: Partial<AppSettings>) => void;
  updateTrayStatus: (status: TrayStatus) => void;
  toggleNotifications: () => void;
  loadSettings: () => void;
}

const defaultSettings: AppSettings = {
  terminal: {
    font_family: 'JetBrains Mono',
    font_size: 15,
    scrollback: 10000,
    theme: 'dark',
    cursor_style: 'block',
    mouse_tracking: true,
    bracketed_paste: true,
    input_batch_delay_ms: 50,
  },
  agents: {
    max_sessions: 50,
    output_buffer_mb: 512,
    auto_focus_on_spawn: true,
    show_agent_list: true,
  },
  notifications: {
    enabled: true,
    sound: true,
    focus_when_focused: true,
    coalesce_window_seconds: 30,
    triggers: ['agent_blocked', 'agent_done', 'agent_error'],
  },
  hotkeys: {
    nav_prev_pane: 'Ctrl+PageUp',
    nav_next_pane: 'Ctrl+PageDown',
    nav_cycle_pane: 'Ctrl+Tab',
    nav_focus_left: 'Ctrl+h',
    nav_focus_down: 'Ctrl+j',
    nav_focus_up: 'Ctrl+k',
    nav_focus_right: 'Ctrl+l',
    nav_split_right: 'Ctrl+Shift+e',
    nav_split_down: 'Ctrl+Shift+o',
    nav_zoom_toggle: 'Ctrl+z',
    nav_close_pane: 'Ctrl+d',
    agent_interrupt: 'Ctrl+c',
    agent_new: 'Ctrl+n',
    window_new: 'Ctrl+Shift+n',
  },
  crash_reporting: {
    enabled: false,
    dsn: '',
  },
};

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: defaultSettings,
  trayStatus: { working: 0, done: 0, blocked: 0, error: 0 },
  notificationsEnabled: true,

  updateSettings: (updates) => set((state) => ({
    settings: { ...state.settings, ...updates },
  })),

  updateTrayStatus: (status) => set({ trayStatus: status }),

  toggleNotifications: () => set((state) => {
    const enabled = !state.notificationsEnabled;
    return { notificationsEnabled: enabled };
  }),

  loadSettings: () => {
    invoke<any>('get_config')
      .then((config: any) => {
        if (config.settings) {
          set({ settings: config.settings });
        }
      })
      .catch(console.error);
  },
}));
