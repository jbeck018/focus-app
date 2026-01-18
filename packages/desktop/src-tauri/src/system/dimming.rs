// system/dimming.rs - Screen dimming overlay system for focus mode
//
// Creates transparent overlay windows to dim all screen content except
// the focused application window, reducing visual distractions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Screen dimming overlay state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimmingState {
    /// Whether dimming is currently enabled
    pub enabled: bool,
    /// Dimming opacity (0.0 = transparent, 1.0 = fully opaque)
    pub opacity: f32,
    /// Strict mode prevents any bypass of dimming
    pub strict_mode: bool,
    /// Session ID this dimming is tied to
    pub session_id: Option<String>,
    /// When dimming was enabled
    pub enabled_at: Option<DateTime<Utc>>,
    /// Window labels for created overlay windows
    pub overlay_window_labels: Vec<String>,
}

impl Default for DimmingState {
    fn default() -> Self {
        Self {
            enabled: false,
            opacity: 0.7,  // 70% dim by default
            strict_mode: true,  // No bypass by default
            session_id: None,
            enabled_at: None,
            overlay_window_labels: Vec::new(),
        }
    }
}

impl DimmingState {
    /// Enable dimming with specified opacity
    pub fn enable(&mut self, opacity: f32, strict: bool, session_id: Option<String>) {
        self.enabled = true;
        self.opacity = opacity.clamp(0.0, 1.0);
        self.strict_mode = strict;
        self.session_id = session_id;
        self.enabled_at = Some(Utc::now());
    }

    /// Disable dimming
    pub fn disable(&mut self) {
        self.enabled = false;
        self.session_id = None;
        self.enabled_at = None;
        self.overlay_window_labels.clear();
    }

    /// Update opacity level
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    /// Add an overlay window label
    pub fn add_overlay(&mut self, label: String) {
        if !self.overlay_window_labels.contains(&label) {
            self.overlay_window_labels.push(label);
        }
    }

    /// Remove an overlay window label
    pub fn remove_overlay(&mut self, label: &str) {
        self.overlay_window_labels.retain(|l| l != label);
    }
}

/// Monitor information for overlay positioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    pub is_primary: bool,
}

/// Rectangle representing a window position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Foreground window information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForegroundWindowInfo {
    pub title: String,
    pub process_name: Option<String>,
    pub rect: WindowRect,
    pub is_focusflow: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimming_state_default() {
        let state = DimmingState::default();
        assert!(!state.enabled);
        assert_eq!(state.opacity, 0.7);
        assert!(state.strict_mode);
        assert!(state.session_id.is_none());
    }

    #[test]
    fn test_dimming_enable_disable() {
        let mut state = DimmingState::default();

        state.enable(0.8, true, Some("session-123".to_string()));
        assert!(state.enabled);
        assert_eq!(state.opacity, 0.8);
        assert!(state.strict_mode);
        assert_eq!(state.session_id, Some("session-123".to_string()));
        assert!(state.enabled_at.is_some());

        state.disable();
        assert!(!state.enabled);
        assert!(state.session_id.is_none());
        assert!(state.enabled_at.is_none());
    }

    #[test]
    fn test_opacity_clamping() {
        let mut state = DimmingState::default();

        state.set_opacity(1.5);
        assert_eq!(state.opacity, 1.0);

        state.set_opacity(-0.5);
        assert_eq!(state.opacity, 0.0);

        state.set_opacity(0.5);
        assert_eq!(state.opacity, 0.5);
    }

    #[test]
    fn test_overlay_management() {
        let mut state = DimmingState::default();

        state.add_overlay("overlay-1".to_string());
        state.add_overlay("overlay-2".to_string());
        state.add_overlay("overlay-1".to_string()); // Duplicate

        assert_eq!(state.overlay_window_labels.len(), 2);

        state.remove_overlay("overlay-1");
        assert_eq!(state.overlay_window_labels.len(), 1);
        assert!(state.overlay_window_labels.contains(&"overlay-2".to_string()));
    }
}
