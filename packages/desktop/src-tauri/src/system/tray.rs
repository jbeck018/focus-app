// system/tray.rs - System tray icon and menu management

use tauri::{
    menu::{Menu, MenuEvent, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
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
pub fn update_tray_menu<R: Runtime>(app: &tauri::AppHandle<R>, session_active: bool) {
    // Get the managed TrayIcon from app state
    let tray = match app.try_state::<TrayIcon<R>>() {
        Some(tray) => tray,
        None => {
            tracing::warn!("Tray icon not found in app state");
            return;
        }
    };

    // Create a new menu with updated enabled states
    // When session is active: disable "Start Focus Session", enable "Stop Session"
    // When session is inactive: enable "Start Focus Session", disable "Stop Session"
    let menu_result = (|| -> tauri::Result<Menu<R>> {
        let show = MenuItem::with_id(app, "show", "Show FocusFlow", true, None::<&str>)?;
        let start_focus = MenuItem::with_id(
            app,
            "start_focus",
            "Start Focus Session",
            !session_active, // Enabled when session is NOT active
            None::<&str>,
        )?;
        let stop_focus = MenuItem::with_id(
            app,
            "stop_focus",
            "Stop Session",
            session_active, // Enabled when session IS active
            None::<&str>,
        )?;
        let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

        Menu::with_items(app, &[&show, &start_focus, &stop_focus, &quit])
    })();

    match menu_result {
        Ok(menu) => {
            if let Err(e) = tray.set_menu(Some(menu)) {
                tracing::error!("Failed to update tray menu: {}", e);
            } else {
                tracing::info!("Tray menu updated: session_active={}", session_active);
            }
        }
        Err(e) => {
            tracing::error!("Failed to create tray menu: {}", e);
        }
    }
}

/// Update tray icon based on application state
///
/// Changes the tooltip to indicate current state:
/// - Idle: "FocusFlow - Idle"
/// - Focus: "FocusFlow - Focus Session Active"
/// - Break: "FocusFlow - Break Time"
///
/// Note: Changing the actual icon requires platform-specific icon assets.
/// For now, we update the tooltip to reflect the current state.
#[allow(dead_code)]
pub fn update_tray_icon<R: Runtime>(
    app: &tauri::AppHandle<R>,
    state: TrayIconState,
) {
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
        TrayIconState::Idle => "FocusFlow - Idle",
        TrayIconState::Focus => "FocusFlow - Focus Session Active",
        TrayIconState::Break => "FocusFlow - Break Time",
    };

    if let Err(e) = tray.set_tooltip(Some(tooltip)) {
        tracing::error!("Failed to update tray tooltip: {}", e);
    } else {
        tracing::debug!("Tray tooltip updated to: {}", tooltip);
    }

    // Note: To change the actual icon, you would need to:
    // 1. Create different icon assets (e.g., icon-idle.png, icon-focus.png, icon-break.png)
    // 2. Use tray.set_icon() with the appropriate icon based on state
    // Example:
    // let icon_path = match state {
    //     TrayIconState::Idle => "icons/icon-idle.png",
    //     TrayIconState::Focus => "icons/icon-focus.png",
    //     TrayIconState::Break => "icons/icon-break.png",
    // };
    // if let Ok(icon) = tauri::image::Image::from_path(icon_path) {
    //     let _ = tray.set_icon(Some(icon));
    // }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum TrayIconState {
    Idle,
    Focus,
    Break,
}
