use serde::de::DeserializeOwned;
use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_store::StoreExt;

use crate::model::{Recent, Settings};

const SETTINGS_FILE: &str = "settings.json";
const RECENT_FILE: &str = "recent.json";
const KEY: &str = "value";

pub(crate) fn read<R: Runtime, T: DeserializeOwned + Default>(app: &AppHandle<R>, file: &str) -> T {
    app.store(file)
        .ok()
        .and_then(|store| store.get(KEY))
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default()
}

pub(crate) fn write<R: Runtime, T: Serialize>(
    app: &AppHandle<R>,
    file: &str,
    value: &T,
) -> Result<(), String> {
    let store = app.store(file).map_err(|err| err.to_string())?;
    let json = serde_json::to_value(value).map_err(|err| err.to_string())?;
    store.set(KEY, json);
    store
        .save()
        .map_err(|err| format!("Could not save {file}: {err}"))
}

fn default_projects_dir<R: Runtime>(app: &AppHandle<R>) -> String {
    let base = app
        .path()
        .document_dir()
        .or_else(|_| app.path().home_dir())
        .unwrap_or_default();
    base.join("Fusible").to_string_lossy().into_owned()
}

/// Loads the settings, filling the projects folder with `Documents/Fusible` when unset.
pub fn load_settings<R: Runtime>(app: &AppHandle<R>) -> Settings {
    let mut settings: Settings = read(app, SETTINGS_FILE);
    if settings.projects_dir.trim().is_empty() {
        settings.projects_dir = default_projects_dir(app);
    }
    settings
}

/// Persists the settings.
pub fn save_settings<R: Runtime>(app: &AppHandle<R>, settings: &Settings) -> Result<(), String> {
    write(app, SETTINGS_FILE, settings)
}

/// Loads the recent list, newest first.
pub fn load_recent<R: Runtime>(app: &AppHandle<R>) -> Vec<Recent> {
    read(app, RECENT_FILE)
}

/// Persists the recent list.
pub fn save_recent<R: Runtime>(app: &AppHandle<R>, list: &[Recent]) -> Result<(), String> {
    write(app, RECENT_FILE, &list)
}
