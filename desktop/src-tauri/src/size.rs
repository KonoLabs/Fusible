use tauri::{AppHandle, LogicalSize, Runtime};

use crate::{storage, window};

const FILE: &str = "window.json";

fn load<R: Runtime>(app: &AppHandle<R>) -> Option<(f64, f64)> {
    storage::read::<R, Option<(f64, f64)>>(app, FILE)
}

/// Applies the remembered window size, if any.
pub fn restore<R: Runtime>(app: &AppHandle<R>) {
    let Some((width, height)) = load(app) else {
        return;
    };
    if let Ok(window) = window::main(app) {
        let _ = window.set_size(LogicalSize::new(width, height));
    }
}

/// Remembers the current window size.
pub fn remember<R: Runtime>(app: &AppHandle<R>) {
    let Ok(window) = window::main(app) else {
        return;
    };
    let (Ok(size), Ok(scale)) = (window.inner_size(), window.scale_factor()) else {
        return;
    };
    let logical = size.to_logical::<f64>(scale);
    let _ = storage::write(app, FILE, &(logical.width, logical.height));
}
