//! The Sync requests and what each one answers, independent of HTTP.

use std::path::Path;
use std::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use super::git_error::Failure;
use super::git_open::{open, Opened};
use super::git_ops::{save, update, Outcome};
use super::git_resolve::{resolve, Keep};
use super::git_run::Git;
use super::git_status::{status, Status};
use super::limiter::lock;

static BUSY: Mutex<()> = Mutex::new(());

const REBUILD_EXTENSIONS: [&str; 7] =
    ["kicad_pcb", "kicad_sch", "tsx", "ts", "yml", "yaml", "json"];

/// One thing the Sync panel can ask for.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Route {
    Status { check: bool },
    Save { message: String },
    Update,
    Resolve { path: String, keep: Keep },
}

#[derive(Deserialize, Default)]
struct Fields {
    message: Option<String>,
    path: Option<String>,
    keep: Option<String>,
}

/// What to send back, and the files whose change should rebuild the boards.
pub(super) struct Reply {
    pub code: u16,
    pub body: Value,
    pub changed: Vec<String>,
}

/// Turns a request path and JSON body into a [`Route`]; the error is the 400 message.
pub(super) fn parse(path: &str, body: &str) -> Result<Route, String> {
    let fields: Fields = if body.trim().is_empty() {
        Fields::default()
    } else {
        serde_json::from_str(body).map_err(|_| "the request is not valid JSON")?
    };
    match path {
        "/api/git/save" => Ok(Route::Save {
            message: fields.message.unwrap_or_default(),
        }),
        "/api/git/update" => Ok(Route::Update),
        "/api/git/resolve" => {
            let keep = fields.keep.as_deref().and_then(Keep::parse);
            match (fields.path, keep) {
                (Some(path), Some(keep)) if !path.is_empty() => Ok(Route::Resolve { path, keep }),
                _ => Err("resolve needs a path and keep: \"mine\" or \"theirs\"".into()),
            }
        }
        _ => Err("unknown request".into()),
    }
}

/// Whether a change to `path` should rebuild the boards.
pub(super) fn is_project_file(path: &str) -> bool {
    let ext = Path::new(path).extension().and_then(|e| e.to_str());
    path.starts_with("hardware/") || ext.is_some_and(|e| REBUILD_EXTENSIONS.contains(&e))
}

fn finish(git: &Git, outcome: Outcome) -> Reply {
    let now = serde_json::to_value(status(git, false)).unwrap_or_default();
    match outcome {
        Ok(done) => Reply {
            code: 200,
            body: json!({ "ok": true, "message": done.message, "status": now }),
            changed: done.changed,
        },
        Err(Failure {
            code,
            error,
            details,
        }) => Reply {
            code,
            body: json!({ "error": error, "details": details, "status": now }),
            changed: Vec::new(),
        },
    }
}

fn plain(code: u16, body: Value) -> Reply {
    Reply {
        code,
        body,
        changed: Vec::new(),
    }
}

fn act(git: &Git, route: Route) -> Reply {
    let _turn = lock(&BUSY);
    let outcome = match route {
        Route::Save { message } => save(git, &message),
        Route::Update => update(git),
        Route::Resolve { path, keep } => resolve(git, &path, keep),
        Route::Status { .. } => return plain(500, json!({ "error": "not an action" })),
    };
    finish(git, outcome)
}

/// Runs `route` with the git binary `program` against the work tree containing `dir`.
pub(super) fn perform(program: &str, dir: &Path, route: Route) -> Reply {
    let dir = if dir.as_os_str().is_empty() {
        Path::new(".")
    } else {
        dir
    };
    match (open(program, dir), route) {
        (Opened::NotRepo, Route::Status { .. }) => plain(200, json!({ "repo": false })),
        (Opened::NotRepo, _) => plain(404, json!({ "error": "This folder is not synced." })),
        (Opened::Missing, Route::Status { .. }) => plain(
            200,
            serde_json::to_value(Status::without_git()).unwrap_or_default(),
        ),
        (Opened::Missing, _) => plain(503, json!({ "error": "Git is not installed." })),
        (Opened::Ready(git), Route::Status { check }) => {
            let _turn = lock(&BUSY);
            let now = status(&git, check);
            plain(200, serde_json::to_value(now).unwrap_or_default())
        }
        (Opened::Ready(git), route) => act(&git, route),
    }
}
