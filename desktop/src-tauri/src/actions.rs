use std::sync::Mutex;

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_opener::OpenerExt;

use crate::{commands::open, window};

const DOCS_URL: &str = "https://github.com/KonoLabs/Fusible#readme";
const BUG_URL: &str = "https://github.com/KonoLabs/Fusible/issues/new";
const ZOOM_STEP: f64 = 0.1;

/// Current page zoom of the main window.
pub struct Zoom(pub Mutex<f64>);

impl Default for Zoom {
    fn default() -> Self {
        Self(Mutex::new(1.0))
    }
}

/// Next zoom factor for a menu action id, clamped to 50% through 300%.
pub fn next_zoom(current: f64, action: &str) -> f64 {
    let next = match action {
        "zoom-in" => current + ZOOM_STEP,
        "zoom-out" => current - ZOOM_STEP,
        _ => 1.0,
    };
    (next * 10.0).round().clamp(5.0, 30.0) / 10.0
}

fn show<R: Runtime>(app: &AppHandle<R>, fragment: Option<&str>) {
    let _ = open::stop_and_show(app, fragment);
}

fn zoom<R: Runtime>(app: &AppHandle<R>, action: &str) {
    let state = app.state::<Zoom>();
    let Ok(mut factor) = state.0.lock() else {
        return;
    };
    *factor = next_zoom(*factor, action);
    if let Ok(window) = window::main(app) {
        let _ = window.set_zoom(*factor);
    }
}

fn reload<R: Runtime>(app: &AppHandle<R>) {
    if let Ok(window) = window::main(app) {
        let _ = window.eval("window.location.reload()");
    }
}

fn open_url<R: Runtime>(app: &AppHandle<R>, url: &str) {
    let _ = app.opener().open_url(url, None::<&str>);
}

/// Runs the action behind a menu item id.
pub fn handle<R: Runtime>(app: &AppHandle<R>, id: &str) {
    match id {
        "projects" => show(app, None),
        "open-folder" => show(app, Some("open")),
        "new-project" => show(app, Some("new")),
        "settings" => show(app, Some("settings")),
        "quit" => app.exit(0),
        "reload" => reload(app),
        "zoom-in" | "zoom-out" | "zoom-reset" => zoom(app, id),
        "docs" => open_url(app, DOCS_URL),
        "bug" => open_url(app, BUG_URL),
        _ => {}
    }
}
