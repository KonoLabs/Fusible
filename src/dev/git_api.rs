//! `GET /api/git` and the JSON POSTs under `/api/git/`.

use std::io::Read;
use std::path::Path;
use std::thread;

use tiny_http::{Method, Request};

use super::api::{error_body, respond_json};
use super::git_route::{is_project_file, parse, perform, Reply, Route};
use super::refresh::refresh;
use super::run_api::forbidden;
use super::server::{query_param, route_path};
use super::state::State;
use crate::config::FusibleConfig;

const MAX_REQUEST: u64 = 4096;

/// Whether `path` is one of the Sync endpoints.
pub(super) fn owns(path: &str) -> bool {
    matches!(
        path,
        "/api/git" | "/api/git/save" | "/api/git/update" | "/api/git/resolve"
    )
}

fn project_dir() -> std::path::PathBuf {
    let found = FusibleConfig::nearest().map(|(path, _)| path);
    found
        .as_deref()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

fn rebuild(state: &State) {
    for slot in state.slots.iter() {
        let (slot, state) = (slot.clone(), state.clone());
        thread::spawn(move || refresh(&slot, &state));
    }
}

fn respond(request: Request, state: &State, reply: Reply) {
    if reply.changed.iter().any(|p| is_project_file(p)) {
        rebuild(state);
    }
    respond_json(request, reply.code, reply.body.to_string());
}

/// Answers the Sync panel: the status on GET, and save, update or resolve on guarded POSTs.
pub(super) fn serve(mut request: Request, state: &State, port: u16) {
    let path = route_path(request.url()).to_string();
    let route = if path == "/api/git" {
        if *request.method() != Method::Get {
            return respond_json(request, 405, error_body("use GET"));
        }
        let check = query_param(request.url(), "check").is_some();
        Ok(Route::Status { check })
    } else if forbidden(&request, port) {
        return respond_json(request, 403, error_body("forbidden"));
    } else {
        let mut body = String::new();
        let read = request
            .as_reader()
            .take(MAX_REQUEST)
            .read_to_string(&mut body);
        match read {
            Ok(_) => parse(&path, &body),
            Err(_) => Err("unreadable request body".into()),
        }
    };
    match route {
        Ok(route) => {
            let reply = perform("git", &project_dir(), route);
            respond(request, state, reply);
        }
        Err(message) => respond_json(request, 400, error_body(&message)),
    }
}
