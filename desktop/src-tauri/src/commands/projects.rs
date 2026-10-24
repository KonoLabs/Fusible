use std::path::{Path, PathBuf};

use tauri::{AppHandle, Runtime};
use tauri_plugin_opener::OpenerExt;

use crate::model::{FolderInfo, Project};
use crate::{discovery, manifest, project, recent, sidecar, storage, validate};

/// Recent projects plus folders with a manifest under the projects folder, recent first.
#[tauri::command]
pub fn list_projects<R: Runtime>(app: AppHandle<R>) -> Result<Vec<Project>, String> {
    let settings = storage::load_settings(&app);
    let found = discovery::find_projects(Path::new(&settings.projects_dir));
    Ok(project::merge(&storage::load_recent(&app), &found))
}

/// Creates a project with `fusible init`, in a new folder named after it or in `parent` itself.
#[tauri::command]
pub async fn create_project<R: Runtime>(
    app: AppHandle<R>,
    parent: String,
    name: String,
    create_folder: bool,
    currency: String,
) -> Result<Project, String> {
    let name = validate::project_name(&name)?;
    let base = PathBuf::from(&parent);
    let dir = if create_folder {
        base.join(&name)
    } else {
        base
    };
    std::fs::create_dir_all(&dir)
        .map_err(|err| format!("Could not create {}: {err}", dir.display()))?;
    let command =
        sidecar::fusible(&app, &dir)?.args(["init", "--name", &name, "--currency", &currency]);
    let output = command
        .output()
        .await
        .map_err(|err| format!("Could not run fusible: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "fusible init failed.".into()
        } else {
            stderr
        });
    }
    Ok(project::describe(&dir, None))
}

/// Tells whether a folder holds a manifest and what its project is called.
#[tauri::command]
pub fn inspect_folder(path: String) -> Result<FolderInfo, String> {
    let summary = manifest::read(Path::new(&path));
    Ok(FolderInfo {
        has_manifest: summary.is_some(),
        name: summary.and_then(|summary| summary.name),
    })
}

/// Removes a project from the recent list. Nothing on disk changes.
#[tauri::command]
pub fn forget_project<R: Runtime>(app: AppHandle<R>, path: String) -> Result<(), String> {
    let key = project::canonical(Path::new(&path));
    let kept: Vec<_> = storage::load_recent(&app)
        .into_iter()
        .filter(|entry| entry.path != path && project::canonical(Path::new(&entry.path)) != key)
        .collect();
    storage::save_recent(&app, &recent::remove(kept, &path))
}

/// Opens the project folder in the OS file manager.
#[tauri::command]
pub fn reveal_project<R: Runtime>(app: AppHandle<R>, path: String) -> Result<(), String> {
    if !Path::new(&path).is_dir() {
        return Err(format!("The folder {path} no longer exists."));
    }
    app.opener()
        .open_path(&path, None::<&str>)
        .map_err(|err| format!("Could not open the folder: {err}"))
}
