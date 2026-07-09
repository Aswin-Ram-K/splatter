//! Hotkey registry and management.

use serde::{Deserialize, Serialize};

/// Global hotkey configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub nav_prev_pane: String,
    pub nav_next_pane: String,
    pub nav_cycle_pane: String,
    pub nav_focus_left: String,
    pub nav_focus_down: String,
    pub nav_focus_up: String,
    pub nav_focus_right: String,
    pub nav_split_right: String,
    pub nav_split_down: String,
    pub nav_zoom_toggle: String,
    pub nav_close_pane: String,
    pub agent_interrupt: String,
    pub agent_new: String,
    pub window_new: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            nav_prev_pane: "Ctrl+PageUp".to_string(),
            nav_next_pane: "Ctrl+PageDown".to_string(),
            nav_cycle_pane: "Ctrl+Tab".to_string(),
            nav_focus_left: "Ctrl+h".to_string(),
            nav_focus_down: "Ctrl+j".to_string(),
            nav_focus_up: "Ctrl+k".to_string(),
            nav_focus_right: "Ctrl+l".to_string(),
            nav_split_right: "Ctrl+Shift+e".to_string(),
            nav_split_down: "Ctrl+Shift+o".to_string(),
            nav_zoom_toggle: "Ctrl+z".to_string(),
            nav_close_pane: "Ctrl+d".to_string(),
            agent_interrupt: "Ctrl+c".to_string(),
            agent_new: "Ctrl+n".to_string(),
            window_new: "Ctrl+Shift+n".to_string(),
        }
    }
}

/// Parse a hotkey string into components.
/// Format: "Ctrl+Shift+e" → (Ctrl, Shift, e)
pub fn parse_hotkey(s: &str) -> Hotkey {
    let mut modifiers = Vec::new();
    let mut key = String::new();

    for part in s.split('+') {
        let part = part.trim();
        match part {
            "Ctrl" | "Control" => modifiers.push(Modifier::Ctrl),
            "Shift" | "ShiftLeft" | "ShiftRight" => modifiers.push(Modifier::Shift),
            "Alt" | "Option" => modifiers.push(Modifier::Alt),
            "Super" | "Meta" | "Win" => modifiers.push(Modifier::Super),
            _ => key = part.to_uppercase(),
        }
    }

    Hotkey { modifiers, key }
}

/// A parsed hotkey.
#[derive(Debug, Clone)]
pub struct Hotkey {
    pub modifiers: Vec<Modifier>,
    pub key: String,
}

/// Modifier keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    Ctrl,
    Shift,
    Alt,
    Super,
}

/// Hotkey action (what to do when triggered).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotkeyAction {
    NavPrevPane,
    NavNextPane,
    NavCyclePane,
    NavFocusLeft,
    NavFocusDown,
    NavFocusUp,
    NavFocusRight,
    NavSplitRight,
    NavSplitDown,
    NavZoomToggle,
    NavClosePane,
    AgentInterrupt,
    AgentNew,
    WindowNew,
}

/// Hotkey registry.
pub struct HotkeyRegistry {
    bindings: Vec<(Hotkey, HotkeyAction)>,
}

impl Default for HotkeyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HotkeyRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            bindings: Vec::new(),
        };
        registry.register_defaults();
        registry
    }

    fn register(&mut self, key_str: &str, action: HotkeyAction) {
        let hk = parse_hotkey(key_str);
        self.bindings.push((hk, action));
    }

    fn register_defaults(&mut self) {
        let defaults = HotkeyConfig::default();
        self.register(&defaults.nav_prev_pane, HotkeyAction::NavPrevPane);
        self.register(&defaults.nav_next_pane, HotkeyAction::NavNextPane);
        self.register(&defaults.nav_cycle_pane, HotkeyAction::NavCyclePane);
        self.register(&defaults.nav_focus_left, HotkeyAction::NavFocusLeft);
        self.register(&defaults.nav_focus_down, HotkeyAction::NavFocusDown);
        self.register(&defaults.nav_focus_up, HotkeyAction::NavFocusUp);
        self.register(&defaults.nav_focus_right, HotkeyAction::NavFocusRight);
        self.register(&defaults.nav_split_right, HotkeyAction::NavSplitRight);
        self.register(&defaults.nav_split_down, HotkeyAction::NavSplitDown);
        self.register(&defaults.nav_zoom_toggle, HotkeyAction::NavZoomToggle);
        self.register(&defaults.nav_close_pane, HotkeyAction::NavClosePane);
        self.register(&defaults.agent_interrupt, HotkeyAction::AgentInterrupt);
        self.register(&defaults.agent_new, HotkeyAction::AgentNew);
        self.register(&defaults.window_new, HotkeyAction::WindowNew);
    }

    /// Look up an action for a given hotkey string.
    pub fn lookup(&self, key_str: &str) -> Option<HotkeyAction> {
        let parsed = parse_hotkey(key_str);
        self.bindings
            .iter()
            .find(|(hk, _)| hk.key == parsed.key && hk.modifiers == parsed.modifiers)
            .map(|(_, action)| action.clone())
    }
}

#[allow(dead_code)]
impl HotkeyConfig {
    #[allow(dead_code)]
    fn nav_nav_prev_pane(s: &Self) -> Hotkey {
        parse_hotkey(&s.nav_prev_pane)
    }
    #[allow(dead_code)]
    fn nav_next_pane(s: &Self) -> Hotkey {
        parse_hotkey(&s.nav_next_pane)
    }
    #[allow(dead_code)]
    fn nav_cycle_pane(s: &Self) -> Hotkey {
        parse_hotkey(&s.nav_cycle_pane)
    }
    #[allow(dead_code)]
    fn nav_focus_left(s: &Self) -> Hotkey {
        parse_hotkey(&s.nav_focus_left)
    }
    #[allow(dead_code)]
    fn nav_focus_down(s: &Self) -> Hotkey {
        parse_hotkey(&s.nav_focus_down)
    }
    #[allow(dead_code)]
    fn nav_focus_up(s: &Self) -> Hotkey {
        parse_hotkey(&s.nav_focus_up)
    }
    #[allow(dead_code)]
    fn nav_focus_right(s: &Self) -> Hotkey {
        parse_hotkey(&s.nav_focus_right)
    }
    fn nav_split_right(s: &Self) -> Hotkey {
        parse_hotkey(&s.nav_split_right)
    }
    fn nav_split_down(s: &Self) -> Hotkey {
        parse_hotkey(&s.nav_split_down)
    }
    fn nav_zoom_toggle(s: &Self) -> Hotkey {
        parse_hotkey(&s.nav_zoom_toggle)
    }
    fn nav_close_pane(s: &Self) -> Hotkey {
        parse_hotkey(&s.nav_close_pane)
    }
    fn agent_interrupt(s: &Self) -> Hotkey {
        parse_hotkey(&s.agent_interrupt)
    }
    fn agent_new(s: &Self) -> Hotkey {
        parse_hotkey(&s.agent_new)
    }
    fn window_new(s: &Self) -> Hotkey {
        parse_hotkey(&s.window_new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hotkey_ctrl_e() {
        let hk = parse_hotkey("Ctrl+e");
        assert_eq!(hk.key, "E");
        assert_eq!(hk.modifiers, vec![Modifier::Ctrl]);
    }

    #[test]
    fn test_parse_hotkey_ctrl_shift_e() {
        let hk = parse_hotkey("Ctrl+Shift+e");
        assert_eq!(hk.key, "E");
        assert_eq!(hk.modifiers, vec![Modifier::Ctrl, Modifier::Shift]);
    }

    #[test]
    fn test_parse_hotkey_ctrl_tab() {
        let hk = parse_hotkey("Ctrl+Tab");
        assert_eq!(hk.key, "TAB");
    }

    #[test]
    fn test_registry_lookup() {
        let registry = HotkeyRegistry::new();
        assert_eq!(registry.lookup("Ctrl+n"), Some(HotkeyAction::AgentNew));
        assert_eq!(
            registry.lookup("Ctrl+c"),
            Some(HotkeyAction::AgentInterrupt)
        );
        assert_eq!(registry.lookup("Ctrl+Shift+x"), None);
    }
}
