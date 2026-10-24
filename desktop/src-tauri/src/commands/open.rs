use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager, Runtime, State};

use crate::studio::Studio;
use crate::{launch, manifest, project, recent, storage, timestamp, window};

fn check_folder(dir: &Path) -> Result<(), String> {
    if !dir.is_dir() {
        return Err(format!("The folder {} no longer exists.", dir.display()));
    }
    if manifest::find(dir).is_none() {
        return Err(format!("{} has no fusible.yml.", dir.display()));
    }
    Ok(())
}

fn remember<R: Runtime>(app: &AppHandle<R>, path: &str) {
    let list = recent::push(storage::load_recent(app), path, &timestamp::now());
    let _ = storage::save_recent(app, &list);
}

/// Starts the studio for the folder, navigates the window to it and records it as recent.
pub async fn open<R: Runtime>(app: &AppHandle<R>, path: &str) -> Result<(), String> {
    let dir = PathBuf::from(path);
    check_folder(&dir)?;
    let studio = app.state::<Studio>();
    let port = storage::load_settings(app).port;
    let ready = launch::start(app, &studio, &dir, port).await?;
    let card = project::describe(&dir, None);
    remember(app, &card.path);
    window::show_studio(app, &ready.url, &card.name)
}

/// Opens a project in the studio, replacing the one that runs.
#[tauri::command]
pub async fn open_project<R: Runtime>(app: AppHandle<R>, path: String) -> Result<(), String> {
    open(&app, &path).await
}

/// Stops the studio and shows the Projects page, optionally at a hash such as `settings`.
pub fn stop_and_show<R: Runtime>(app: &AppHandle<R>, fragment: Option<&str>) -> Result<(), String> {
    app.state::<Studio>().stop();
    window::show_projects(app, fragment)
}

/// Stops the studio and returns to the Projects page.
#[tauri::command]
pub fn open_projects<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    stop_and_show(&app, None)
}

/// The stderr of the last failed open, if any.
#[tauri::command]
pub fn last_error(studio: State<'_, Studio>) -> Option<String> {
    studio.last_error()
}
