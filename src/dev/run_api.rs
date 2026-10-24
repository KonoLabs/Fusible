//! `POST /api/run`: starts a check, build, or body run.

use std::io::Read;
use std::process::Command;

use tiny_http::{Method, Request};

use super::api::{error_body, respond_json, JSON};
use super::guard::is_allowed_origin;
use super::run_request::parse;
use super::runs::Busy;
use super::state::State;
use crate::config::FusibleConfig;

const MAX_REQUEST: u64 = 4096;

pub(super) fn forbidden(request: &Request, port: u16) -> bool {
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
    let read = request
        .as_reader()
        .take(MAX_REQUEST)
        .read_to_string(&mut body);
    if read.is_err() {
        return respond_json(request, 400, error_body("unreadable request body"));
    }
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
