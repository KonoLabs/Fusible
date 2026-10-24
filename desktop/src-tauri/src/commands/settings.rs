use tauri::{AppHandle, Runtime};

use crate::model::Settings;
use crate::{storage, validate};

/// The saved settings, with defaults for anything unset.
#[tauri::command]
pub fn get_settings<R: Runtime>(app: AppHandle<R>) -> Result<Settings, String> {
    Ok(storage::load_settings(&app))
}

/// Validates and saves the settings, returning the saved copy.
#[tauri::command]
pub fn set_settings<R: Runtime>(app: AppHandle<R>, settings: Settings) -> Result<Settings, String> {
    let clean = validate::settings(settings)?;
    storage::save_settings(&app, &clean)?;
    Ok(clean)
}
