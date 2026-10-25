use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::studio::Studio;
use crate::updates::{
    due, supported, Progress, Staged, UpdateCheck, Updates, PROGRESS_EVENT, RELEASES_URL,
};
use crate::{storage, timestamp};

fn describe(update: &Update) -> UpdateCheck {
    UpdateCheck::Available {
        version: update.version.clone(),
        notes: update.body.clone().unwrap_or_default(),
        date: update
            .date
            .map(|date| timestamp::rfc3339(date.unix_timestamp())),
    }
}

fn skip_automatic<R: Runtime>(app: &AppHandle<R>) -> bool {
    let settings = storage::load_settings(app);
    let first = !app
        .state::<Updates>()
        .attempted
        .swap(true, Ordering::SeqCst);
    !(first && settings.auto_update && due(settings.last_update_check, timestamp::unix_now()))
}

fn record_check<R: Runtime>(app: &AppHandle<R>) {
    let mut settings = storage::load_settings(app);
    settings.last_update_check = Some(timestamp::unix_now());
    let _ = storage::save_settings(app, &settings);
}

async fn look<R: Runtime>(app: &AppHandle<R>) -> Result<UpdateCheck, String> {
    let updater = app.updater().map_err(|err| err.to_string())?;
    let found = updater.check().await.map_err(|err| err.to_string())?;
    let result = found.as_ref().map_or(UpdateCheck::UpToDate, describe);
    *app.state::<Updates>().pending.lock().unwrap() = found;
    Ok(result)
}

fn remembered<R: Runtime>(app: &AppHandle<R>) -> Option<UpdateCheck> {
    let pending = app.state::<Updates>().pending.lock().unwrap().clone();
    pending.as_ref().map(describe)
}

/// Looks for a newer signed release. With `auto`, it runs only when the setting is on and a day has passed.
#[tauri::command]
pub async fn check_update<R: Runtime>(
    app: AppHandle<R>,
    auto: bool,
) -> Result<UpdateCheck, String> {
    if !supported(
        cfg!(target_os = "linux"),
        std::env::var_os("APPIMAGE").is_some(),
    ) {
        return Ok(UpdateCheck::Unsupported {
            url: RELEASES_URL.into(),
        });
    }
    if auto {
        if let Some(known) = remembered(&app) {
            return Ok(known);
        }
        if skip_automatic(&app) {
            return Ok(UpdateCheck::Skipped);
        }
    }
    let result = look(&app).await?;
    record_check(&app);
    Ok(result)
}

/// Downloads the update found by `check_update` and verifies its signature, then installs it (Windows: on restart).
#[tauri::command]
pub async fn install_update<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let update = app
        .state::<Updates>()
        .pending
        .lock()
        .unwrap()
        .clone()
        .ok_or("Check for updates first.")?;
    let mut downloaded = 0u64;
    let events = app.clone();
    let bytes = update
        .download(
            move |chunk, total| {
                downloaded += chunk as u64;
                let _ = events.emit(PROGRESS_EVENT, Progress { downloaded, total });
            },
            || {},
        )
        .await
        .map_err(|err| format!("Could not download the update: {err}"))?;
    if cfg!(windows) {
        *app.state::<Updates>().staged.lock().unwrap() = Some(Staged { update, bytes });
        return Ok(());
    }
    update
        .install(&bytes)
        .map_err(|err| format!("Could not install the update: {err}"))
}

/// Stops the studio, applies a staged update, and relaunches the app.
#[tauri::command]
pub fn restart_app<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let staged = app.state::<Updates>().staged.lock().unwrap().take();
    app.state::<Studio>().stop();
    if let Some(Staged { update, bytes }) = staged {
        update
            .install(&bytes)
            .map_err(|err| format!("Could not install the update: {err}"))?;
    }
    app.restart()
}
