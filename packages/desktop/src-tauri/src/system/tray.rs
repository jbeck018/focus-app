// system/tray.rs - System tray icon and menu management

use tauri::{
    menu::{Menu, MenuEvent, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Emitter, Manager, Runtime,
};

/// Setup system tray with dynamic menu
pub fn setup_tray<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
    let menu = create_tray_menu(app)?;

    let tray = TrayIconBuilder::new()
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
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
        .build(app)?;

    // Store tray in app state for later updates
    app.manage(tray);

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
#[allow(dead_code)]
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
        "stop_focus" => {
            // Emit event to frontend to stop current session
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("tray-stop-focus", ());
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

/// Update tray menu based on session state
///
/// Call this when session starts/stops to enable/disable menu items
#[allow(dead_code)]
pub fn update_tray_menu<R: Runtime>(_app: &tauri::AppHandle<R>, session_active: bool) {
    // Get existing menu items and update their enabled state
    // This is a simplified example - in practice you'd rebuild the menu
    // with updated states or use MenuItem::set_enabled() if available

    tracing::info!("Tray menu updated: session_active={}", session_active);

    // TODO: Use Tauri 2.0 API to update menu item states
    // For now, this is a placeholder for the pattern
}

/// Update tray icon based on application state
///
/// Changes icon to indicate:
/// - Default: Gray (idle)
/// - Active session: Green
/// - Break: Yellow
#[allow(dead_code)]
pub fn update_tray_icon<R: Runtime>(
    _app: &tauri::AppHandle<R>,
    _state: TrayIconState,
) {
    // TODO: Load different icon based on state
    // This requires multiple icon assets and platform-specific handling
    tracing::debug!("Tray icon update requested");
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum TrayIconState {
    Idle,
    Focus,
    Break,
}
