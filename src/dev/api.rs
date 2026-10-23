//! JSON endpoints of the dev server.

use std::io::Read;
use std::path::Path;
use std::process::Command;

use tiny_http::{Method, Request};

use super::assets::http_response;
use super::project::project_json;
use super::run_request::parse;
use super::runs::Busy;
use super::server::{is_allowed_origin, query_param};
use super::state::{Slot, State};
use crate::config::FusibleConfig;

const JSON: &str = "application/json";
const MAX_REQUEST: u64 = 4096;

pub(super) fn respond_json(request: Request, status: u16, body: String) {
    let _ = request.respond(http_response(status, JSON, body.into_bytes()));
}

fn error_body(message: &str) -> String {
    serde_json::json!({ "error": message }).to_string()
}

/// Serves the latest payload of the board named by `?board=`.
pub(super) fn circuit(request: Request, state: &State) {
    let board = query_param(request.url(), "board");
    let payload = state.slot(board.as_deref()).map(Slot::payload);
    respond_json(
        request,
        200,
        payload
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| "{}".into()),
    );
}

/// Serves the generated body preview of the board named by `?board=`.
pub(super) fn body_preview(request: Request, state: &State, glb: bool) {
    let board = query_param(request.url(), "board");
    match state.slot(board.as_deref()) {
        Some(slot) => slot.body.serve(request, glb),
        None => respond_json(request, 404, error_body("unknown board")),
    }
}

/// Serves the project tree.
pub(super) fn project(request: Request, state: &State) {
    let (path, cfg) = FusibleConfig::nearest().unzip();
    let root = path
        .as_deref()
        .and_then(Path::parent)
        .unwrap_or(Path::new(""));
    respond_json(request, 200, project_json(state, cfg.as_ref(), root));
}

/// Serves the recent runs with their logs.
pub(super) fn runs(request: Request, state: &State) {
    respond_json(request, 200, state.runs.snapshot());
}

fn forbidden(request: &Request, port: u16) -> bool {
    let header = |name: &'static str| {
        let found = request.headers().iter().find(|h| h.field.equiv(name));
        found.map(|h| h.value.as_str().to_string())
    };
    let json = header("Content-Type").is_some_and(|t| t.starts_with(JSON));
    *request.method() != Method::Post
        || !json
        || !is_allowed_origin(header("Origin").as_deref(), port)
}

/// Starts a check, build, or body run after validating the origin and the target.
pub(super) fn start_run(mut request: Request, state: &State, port: u16) {
    if forbidden(&request, port) {
        return respond_json(request, 403, error_body("forbidden"));
    }
    let mut body = String::new();
    let _ = request
        .as_reader()
        .take(MAX_REQUEST)
        .read_to_string(&mut body);
    let cfg = FusibleConfig::nearest().map(|(_, cfg)| cfg);
    let spec = match parse(&body, cfg.as_ref()) {
        Ok(spec) => spec,
        Err(message) => return respond_json(request, 400, error_body(&message)),
    };
    let Ok(exe) = std::env::current_exe() else {
        return respond_json(request, 500, error_body("cannot locate the fusible binary"));
    };
    let mut command = Command::new(exe);
    command.args(&spec.args);
    match state.runs.start(&state.hub, spec.label, command) {
        Ok(id) => respond_json(request, 202, serde_json::json!({ "run": id }).to_string()),
        Err(Busy) => respond_json(request, 409, error_body("a run is already in progress")),
    }
}
