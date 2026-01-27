// system/tray.rs - System tray icon and menu management
//
// This module manages the system tray icon state to visually indicate:
// - Idle: No active focus session (default app icon)
// - Focus: Active focus session (green-tinted icon with checkmark)
// - Break: Break time (blue-tinted icon)
// - Paused: Session is paused (yellow/orange icon with pause indicator)

use tauri::{
    image::Image,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    App, Emitter, Manager, Runtime,
};
use std::sync::atomic::{AtomicU8, Ordering};

/// Setup system tray with dynamic menu
pub fn setup_tray<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
    let menu = create_tray_menu(app)?;

    let tray = TrayIconBuilder::new()
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .show_menu_on_left_click(false) // Show menu on right click only
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .on_menu_event(|app, event| {
            handle_menu_event(app, event);
        })
        .build(app)?;

    // Store tray in app state for later updates
    app.manage(tray);

    // Initialize tray state to Idle
    set_current_tray_state(TrayIconState::Idle);

    tracing::info!("System tray initialized");

    Ok(())
}

/// Create tray menu with all items
fn create_tray_menu<R: Runtime>(app: &App<R>) -> tauri::Result<Menu<R>> {
    let show = MenuItem::with_id(app, "show", "Show FocusFlow", true, None::<&str>)?;
    let start_focus = MenuItem::with_id(app, "start_focus", "Start Focus Session", true, None::<&str>)?;
    let stop_focus = MenuItem::with_id(app, "stop_focus", "Stop Session", false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show, &start_focus, &stop_focus, &quit])?;

    Ok(menu)
}

/// Handle tray menu events - should be called during app setup
pub fn handle_menu_event<R: Runtime>(app: &tauri::AppHandle<R>, event: MenuEvent) {
    match event.id().as_ref() {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "start_focus" => {
            // Emit event to frontend to show start session dialog
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("tray-start-focus", ());
                let _ = window.show();
            }
        }
        "pause_resume" => {
            // Emit event to frontend to toggle pause/resume
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("tray-toggle-pause", ());
            }
        }
        "stop_focus" => {
            // Emit event to frontend to stop current session
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("tray-stop-focus", ());
            }
        }
        "status" => {
            // Status is not clickable, but we can show the main window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

/// Update tray menu based on session state (legacy compatibility)
///
/// Call this when session starts/stops to enable/disable menu items.
/// Prefer using `update_tray_icon()` with `TrayIconState` for full state management.
pub fn update_tray_menu<R: Runtime>(app: &tauri::AppHandle<R>, session_active: bool) {
    let state = if session_active {
        TrayIconState::Focus
    } else {
        TrayIconState::Idle
    };
    update_tray_menu_for_state(app, state);
}

/// Update tray icon based on application state
///
/// Changes both the tooltip and icon to indicate current state:
/// - Idle: "FocusFlow - Ready" (default icon)
/// - Focus: "FocusFlow - Focus Session Active" (green badge)
/// - Break: "FocusFlow - Break Time" (blue badge)
/// - Paused: "FocusFlow - Session Paused" (yellow badge)
///
/// The icon is modified programmatically using RGBA manipulation to add
/// colored badges/indicators without requiring separate icon files.
pub fn update_tray_icon<R: Runtime>(
    app: &tauri::AppHandle<R>,
    state: TrayIconState,
) {
    // Skip if state hasn't changed
    if get_current_tray_state() == state {
        tracing::trace!("Tray state unchanged, skipping update");
        return;
    }

    // Get the managed TrayIcon from app state
    let tray = match app.try_state::<TrayIcon<R>>() {
        Some(tray) => tray,
        None => {
            tracing::warn!("Tray icon not found in app state");
            return;
        }
    };

    // Update tooltip based on state
    let tooltip = match state {
        TrayIconState::Idle => "FocusFlow - Ready",
        TrayIconState::Focus => "FocusFlow - Focus Session Active",
        TrayIconState::Break => "FocusFlow - Break Time",
        TrayIconState::Paused => "FocusFlow - Session Paused",
    };

    if let Err(e) = tray.set_tooltip(Some(tooltip)) {
        tracing::error!("Failed to update tray tooltip: {}", e);
    }

    // Update the icon with a state indicator badge
    if let Some(base_icon) = app.default_window_icon() {
        match create_tray_icon_with_badge(base_icon, state) {
            Ok(badged_icon) => {
                if let Err(e) = tray.set_icon(Some(badged_icon)) {
                    tracing::error!("Failed to update tray icon: {}", e);
                } else {
                    set_current_tray_state(state);
                    tracing::info!("Tray icon updated to state: {:?}", state);
                }
            }
            Err(e) => {
                tracing::error!("Failed to create badged icon: {}", e);
            }
        }
    }

    // Also update the menu to reflect current state
    update_tray_menu_for_state(app, state);
}

/// Create a tray icon with a colored badge indicator
///
/// Adds a small colored circle in the bottom-right corner of the icon:
/// - Idle: No badge (original icon)
/// - Focus: Green badge
/// - Break: Blue badge
/// - Paused: Yellow/Orange badge
fn create_tray_icon_with_badge(
    base_icon: &Image<'_>,
    state: TrayIconState,
) -> Result<Image<'static>, String> {
    let rgba = base_icon.rgba();
    let width = base_icon.width();
    let height = base_icon.height();

    // Clone the pixel data so we can modify it
    let mut pixels = rgba.to_vec();

    // For idle state, return the original icon
    if state == TrayIconState::Idle {
        return Ok(Image::new_owned(pixels, width, height));
    }

    // Badge color based on state (RGBA)
    let badge_color: [u8; 4] = match state {
        TrayIconState::Idle => [0, 0, 0, 0], // Transparent (won't be used)
        TrayIconState::Focus => [76, 175, 80, 255], // Green (#4CAF50)
        TrayIconState::Break => [33, 150, 243, 255], // Blue (#2196F3)
        TrayIconState::Paused => [255, 152, 0, 255], // Orange (#FF9800)
    };

    // Badge size and position (bottom-right corner)
    // Use percentage-based sizing for different icon resolutions
    let badge_radius = (width.min(height) as f32 * 0.18) as u32; // 18% of smallest dimension
    let badge_center_x = width - badge_radius - (width as f32 * 0.08) as u32;
    let badge_center_y = height - badge_radius - (height as f32 * 0.08) as u32;

    // Draw the badge circle with anti-aliasing
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - badge_center_x as f32;
            let dy = y as f32 - badge_center_y as f32;
            let distance = (dx * dx + dy * dy).sqrt();

            // Inside the badge circle
            if distance <= badge_radius as f32 {
                let pixel_index = ((y * width + x) * 4) as usize;
                if pixel_index + 3 < pixels.len() {
                    // Anti-aliasing at the edge
                    let alpha = if distance > (badge_radius as f32 - 1.0) {
                        let edge_factor = badge_radius as f32 - distance;
                        (edge_factor * 255.0).min(255.0).max(0.0) as u8
                    } else {
                        255
                    };

                    // Blend badge color with alpha
                    if alpha > 0 {
                        let blend_alpha = alpha as f32 / 255.0;
                        pixels[pixel_index] = ((badge_color[0] as f32 * blend_alpha) +
                            (pixels[pixel_index] as f32 * (1.0 - blend_alpha))) as u8;
                        pixels[pixel_index + 1] = ((badge_color[1] as f32 * blend_alpha) +
                            (pixels[pixel_index + 1] as f32 * (1.0 - blend_alpha))) as u8;
                        pixels[pixel_index + 2] = ((badge_color[2] as f32 * blend_alpha) +
                            (pixels[pixel_index + 2] as f32 * (1.0 - blend_alpha))) as u8;
                        // Preserve original alpha for transparency
                        pixels[pixel_index + 3] = pixels[pixel_index + 3].max(alpha);
                    }
                }
            }
        }
    }

    Ok(Image::new_owned(pixels, width, height))
}

/// Update tray menu items based on current state
fn update_tray_menu_for_state<R: Runtime>(app: &tauri::AppHandle<R>, state: TrayIconState) {
    let tray = match app.try_state::<TrayIcon<R>>() {
        Some(tray) => tray,
        None => return,
    };

    let session_active = matches!(state, TrayIconState::Focus | TrayIconState::Break | TrayIconState::Paused);
    let is_paused = matches!(state, TrayIconState::Paused);

    let menu_result = (|| -> tauri::Result<Menu<R>> {
        let show = MenuItem::with_id(app, "show", "Show FocusFlow", true, None::<&str>)?;

        // Add separator after Show
        let separator1 = PredefinedMenuItem::separator(app)?;

        // Session controls - dynamic based on state
        let start_focus = MenuItem::with_id(
            app,
            "start_focus",
            "Start Focus Session",
            !session_active, // Enabled when NO session is active
            None::<&str>,
        )?;

        // Pause/Resume toggle
        let pause_resume_label = if is_paused { "Resume Session" } else { "Pause Session" };
        let pause_resume = MenuItem::with_id(
            app,
            "pause_resume",
            pause_resume_label,
            session_active, // Enabled when session IS active
            None::<&str>,
        )?;

        let stop_focus = MenuItem::with_id(
            app,
            "stop_focus",
            "End Session",
            session_active, // Enabled when session IS active
            None::<&str>,
        )?;

        // Status indicator (non-clickable)
        let status_text = match state {
            TrayIconState::Idle => "Status: Ready",
            TrayIconState::Focus => "Status: Focusing...",
            TrayIconState::Break => "Status: On Break",
            TrayIconState::Paused => "Status: Paused",
        };
        let status = MenuItem::with_id(app, "status", status_text, false, None::<&str>)?;

        let separator2 = PredefinedMenuItem::separator(app)?;
        let quit = MenuItem::with_id(app, "quit", "Quit FocusFlow", true, None::<&str>)?;

        Menu::with_items(app, &[
            &show,
            &separator1,
            &status,
            &start_focus,
            &pause_resume,
            &stop_focus,
            &separator2,
            &quit,
        ])
    })();

    if let Ok(menu) = menu_result {
        if let Err(e) = tray.set_menu(Some(menu)) {
            tracing::error!("Failed to update tray menu for state: {}", e);
        }
    }
}

/// Tray icon state representing the current application status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayIconState {
    /// No active session - default state
    Idle,
    /// Active focus session
    Focus,
    /// Break time between sessions
    Break,
    /// Session is currently paused
    Paused,
}

impl TrayIconState {
    /// Convert to u8 for atomic storage
    fn to_u8(self) -> u8 {
        match self {
            TrayIconState::Idle => 0,
            TrayIconState::Focus => 1,
            TrayIconState::Break => 2,
            TrayIconState::Paused => 3,
        }
    }

    /// Convert from u8
    fn from_u8(value: u8) -> Self {
        match value {
            1 => TrayIconState::Focus,
            2 => TrayIconState::Break,
            3 => TrayIconState::Paused,
            _ => TrayIconState::Idle,
        }
    }
}

/// Global atomic state for current tray icon (for quick checks without locks)
static CURRENT_TRAY_STATE: AtomicU8 = AtomicU8::new(0);

/// Get the current tray icon state
pub fn get_current_tray_state() -> TrayIconState {
    TrayIconState::from_u8(CURRENT_TRAY_STATE.load(Ordering::SeqCst))
}

/// Set the current tray icon state (internal)
fn set_current_tray_state(state: TrayIconState) {
    CURRENT_TRAY_STATE.store(state.to_u8(), Ordering::SeqCst);
}
