//! JSON endpoints of the dev server.

use std::path::Path;

use tiny_http::Request;

use super::assets::http_response;
use super::project::project_json;
use super::server::query_param;
use super::state::State;
use crate::config::FusibleConfig;

pub(super) const JSON: &str = "application/json";

pub(super) fn respond_json(request: Request, status: u16, body: String) {
    let _ = request.respond(http_response(status, JSON, body.into_bytes()));
}

pub(super) fn error_body(message: &str) -> String {
    serde_json::json!({ "error": message }).to_string()
}

/// The status and body for the payload of `board`; `{}` while its first build is running.
pub(super) fn circuit_reply(state: &State, board: Option<&str>) -> (u16, String) {
    match state.slot(board) {
        Some(slot) => (
            200,
            Some(slot.payload())
                .filter(|p| !p.is_empty())
                .unwrap_or_else(|| "{}".into()),
        ),
        None => (404, error_body("unknown board")),
    }
}

/// Serves the latest payload of the board named by `?board=`.
pub(super) fn circuit(request: Request, state: &State) {
    let board = query_param(request.url(), "board");
    let (status, body) = circuit_reply(state, board.as_deref());
    respond_json(request, status, body);
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
