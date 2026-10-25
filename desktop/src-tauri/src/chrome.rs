use tauri::{App, Builder, Wry};

#[cfg(not(target_os = "macos"))]
use crate::window;

/// Removes the native title bar and menu bar on Windows and Linux, where the app draws its own controls.
#[cfg(not(target_os = "macos"))]
pub fn strip(app: &App) {
    if let Ok(window) = window::main(app.handle()) {
        let _ = window.set_decorations(false);
        let _ = window.remove_menu();
    }
}

/// Keeps the native traffic lights and title bar overlay on macOS.
#[cfg(target_os = "macos")]
pub fn strip(_app: &App) {}

/// Attaches the native menu on macOS, where it lives in the system menu bar.
#[cfg(target_os = "macos")]
pub fn with_menu(builder: Builder<Wry>) -> Builder<Wry> {
    builder.menu(crate::menu::build)
}

/// Leaves the window without a menu bar on Windows and Linux.
#[cfg(not(target_os = "macos"))]
pub fn with_menu(builder: Builder<Wry>) -> Builder<Wry> {
    builder
}
