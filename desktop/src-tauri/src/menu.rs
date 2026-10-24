use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Runtime};

fn item<R: Runtime>(
    app: &AppHandle<R>,
    id: &str,
    text: &str,
    keys: Option<&str>,
) -> tauri::Result<MenuItem<R>> {
    MenuItem::with_id(app, id, text, true, keys)
}

fn file_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Submenu<R>> {
    Submenu::with_items(
        app,
        "File",
        true,
        &[
            &item(app, "projects", "Projects", Some("CmdOrCtrl+Shift+O"))?,
            &item(app, "open-folder", "Open folder...", Some("CmdOrCtrl+O"))?,
            &item(app, "new-project", "New project...", Some("CmdOrCtrl+N"))?,
            &item(app, "settings", "Settings", Some("CmdOrCtrl+,"))?,
            &PredefinedMenuItem::separator(app)?,
            &item(app, "quit", "Quit", Some("CmdOrCtrl+Q"))?,
        ],
    )
}

fn edit_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Submenu<R>> {
    Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )
}

fn view_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Submenu<R>> {
    Submenu::with_items(
        app,
        "View",
        true,
        &[
            &item(app, "reload", "Reload", Some("CmdOrCtrl+R"))?,
            &PredefinedMenuItem::separator(app)?,
            &item(app, "zoom-in", "Zoom in", Some("CmdOrCtrl+Plus"))?,
            &item(app, "zoom-out", "Zoom out", Some("CmdOrCtrl+-"))?,
            &item(app, "zoom-reset", "Actual size", Some("CmdOrCtrl+0"))?,
        ],
    )
}

fn help_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Submenu<R>> {
    Submenu::with_items(
        app,
        "Help",
        true,
        &[
            &item(app, "docs", "Fusible docs", None)?,
            &item(app, "bug", "Report a bug", None)?,
            &PredefinedMenuItem::about(app, Some("About Fusible Studio"), None)?,
        ],
    )
}

/// Builds the native menu: File, Edit, View and Help.
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    Menu::with_items(
        app,
        &[
            &file_menu(app)?,
            &edit_menu(app)?,
            &view_menu(app)?,
            &help_menu(app)?,
        ],
    )
}
