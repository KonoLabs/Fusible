use tauri::{App, Builder, Manager, Runtime, Window, WindowEvent, Wry};

use crate::actions::{self, Zoom};
use crate::commands::{open, projects, settings, toolchain};
use crate::studio::Studio;
use crate::window::{self, Home};
use crate::{menu, size, storage};

fn focus_existing<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Ok(window) = window::main(app) {
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn open_last(app: &App) {
    let handle = app.handle().clone();
    if !storage::load_settings(&handle).open_last_project {
        return;
    }
    let last = storage::load_recent(&handle)
        .into_iter()
        .find(|entry| std::path::Path::new(&entry.path).is_dir());
    if let Some(entry) = last {
        tauri::async_runtime::spawn(async move {
            let _ = open::open(&handle, &entry.path).await;
        });
    }
}

fn on_ready(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    size::restore(app.handle());
    open_last(app);
    Ok(())
}

fn plugins(builder: Builder<Wry>) -> Builder<Wry> {
    builder
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            focus_existing(app)
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(Studio::default())
        .manage(Zoom::default())
        .manage(Home::new())
}

fn on_window_event(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { .. } = event {
        size::remember(window.app_handle());
    }
}

/// The configured Tauri builder: plugins, state, menu, commands and the startup hook.
pub fn builder() -> Builder<Wry> {
    plugins(Builder::default())
        .menu(menu::build)
        .on_menu_event(|app, event| actions::handle(app, event.id().as_ref()))
        .on_window_event(on_window_event)
        .invoke_handler(tauri::generate_handler![
            projects::list_projects,
            open::open_project,
            projects::create_project,
            projects::inspect_folder,
            projects::forget_project,
            projects::reveal_project,
            settings::get_settings,
            settings::set_settings,
            toolchain::toolchain_status,
            open::open_projects,
            open::last_error,
        ])
        .setup(on_ready)
}
