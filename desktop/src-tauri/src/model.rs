use serde::{Deserialize, Serialize};

pub const RECENT_LIMIT: usize = 20;
pub const DEFAULT_PORT: u16 = 3020;

/// A project card as the Projects page shows it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub path: String,
    pub name: String,
    pub boards: usize,
    pub firmware: usize,
    pub bodies: usize,
    pub last_opened: Option<String>,
    pub missing: bool,
}

/// User settings persisted in `settings.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub projects_dir: String,
    pub default_currency: String,
    pub port: u16,
    pub open_last_project: bool,
    pub theme: String,
    pub auto_update: bool,
    pub last_update_check: Option<i64>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            projects_dir: String::new(),
            default_currency: "EUR".into(),
            port: DEFAULT_PORT,
            open_last_project: false,
            theme: "system".into(),
            auto_update: true,
            last_update_check: None,
        }
    }
}

/// Answer of `inspect_folder`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FolderInfo {
    pub has_manifest: bool,
    pub name: Option<String>,
}

/// One entry of `recent.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recent {
    pub path: String,
    pub last_opened: String,
}
