use tauri::ipc::CapabilityBuilder;
use tauri::{AppHandle, Manager, Runtime, Url};

use crate::studio::Studio;

const FRAGMENTS: [&str; 3] = ["new", "open", "settings"];

/// Accepts no fragment or one of `new`, `open`, `settings`; anything else is an error.
pub fn check_fragment(fragment: Option<&str>) -> Result<Option<&str>, String> {
    match fragment {
        None => Ok(None),
        Some(name) if FRAGMENTS.contains(&name) => Ok(Some(name)),
        Some(name) => Err(format!("Unknown Projects page section: {name}.")),
    }
}

/// The URL pattern that matches every page of the studio served at `url`.
pub fn studio_pattern(url: &Url) -> String {
    format!("{}/*", url.origin().ascii_serialization())
}

pub fn grant_studio<R: Runtime>(app: &AppHandle<R>, url: &Url) -> Result<(), String> {
    let pattern = studio_pattern(url);
    if !app.state::<Studio>().mark_granted(&pattern) {
        return Ok(());
    }
    let capability = CapabilityBuilder::new(format!(
        "studio-{}",
        url.port_or_known_default().unwrap_or(0)
    ))
    .local(false)
    .remote(pattern)
    .window(crate::window::MAIN)
    .permission("allow-open-projects")
    .permission("core:window:allow-start-dragging")
    .permission("core:window:allow-close")
    .permission("core:window:allow-minimize")
    .permission("core:window:allow-toggle-maximize")
    .permission("core:window:allow-internal-toggle-maximize");
    app.add_capability(capability)
        .map_err(|err| err.to_string())
}
