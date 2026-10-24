use tauri::{AppHandle, Manager, Runtime, Url, WebviewWindow};

pub const MAIN: &str = "main";
const TITLE: &str = "Fusible Studio";

/// The Projects page URL, as the webview of this platform serves the app assets.
pub struct Home(pub Url);

impl Home {
    /// Builds the home URL for the current platform.
    pub fn new() -> Self {
        let base = if cfg!(windows) {
            "http://tauri.localhost/"
        } else {
            "tauri://localhost/"
        };
        Self(Url::parse(base).expect("static url"))
    }
}

impl Default for Home {
    fn default() -> Self {
        Self::new()
    }
}

/// The main window, or an error naming what is missing.
pub fn main<R: Runtime>(app: &AppHandle<R>) -> Result<WebviewWindow<R>, String> {
    app.get_webview_window(MAIN)
        .ok_or_else(|| "The main window is not available.".to_string())
}

/// Window title for a project, or the bare app title when `name` is `None`.
pub fn title_for(name: Option<&str>) -> String {
    match name {
        Some(name) => format!("{name} - {TITLE}"),
        None => TITLE.to_string(),
    }
}

/// Navigates to the Projects page, optionally with a hash such as `settings`.
pub fn show_projects<R: Runtime>(app: &AppHandle<R>, fragment: Option<&str>) -> Result<(), String> {
    let window = main(app)?;
    let mut url = app.state::<Home>().0.clone();
    url.set_fragment(fragment);
    window.navigate(url).map_err(|err| err.to_string())?;
    window
        .set_title(&title_for(None))
        .map_err(|err| err.to_string())
}

/// Navigates to the running studio and titles the window after the project.
pub fn show_studio<R: Runtime>(app: &AppHandle<R>, url: &str, name: &str) -> Result<(), String> {
    let window = main(app)?;
    let target = Url::parse(url).map_err(|err| format!("The studio sent a bad URL: {err}"))?;
    window.navigate(target).map_err(|err| err.to_string())?;
    window
        .set_title(&title_for(Some(name)))
        .map_err(|err| err.to_string())
}
