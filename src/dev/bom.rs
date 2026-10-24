//! `GET /api/bom` and `POST /api/bom`: the priced parts list and its editable file.

use std::io::Read;

use serde_json::json;
use tiny_http::{Method, Request};

use super::api::{error_body, respond_json};
use super::run_api::forbidden;
use super::state::State;
use crate::bom::{self, BomError, BomFile, Part};
use crate::config::FusibleConfig;

const MAX_REQUEST: u64 = 256 * 1024;

fn parts_of(state: &State) -> (Vec<Part>, usize) {
    let scoped = state.slots.len() > 1;
    let mut parts = Vec::new();
    let mut pending = 0;
    for slot in state.slots.iter() {
        let scope = slot.name.as_deref().filter(|_| scoped);
        match slot.source().map(|source| bom::detect(&source, scope)) {
            Some(Ok(found)) => parts.extend(found),
            _ => pending += 1,
        }
    }
    (parts, pending)
}

fn reply(state: &State, file: &BomFile, editable: bool) -> String {
    let (parts, pending) = parts_of(state);
    let (manifest, cfg) = FusibleConfig::nearest().unzip();
    let root = manifest.as_deref().and_then(std::path::Path::parent);
    let cfg = cfg.unwrap_or_default();
    let names: Vec<&str> = state
        .slots
        .iter()
        .filter_map(|s| s.name.as_deref())
        .collect();
    let report = bom::build(
        parts,
        &bom::bodies_of(&cfg, &names, root.unwrap_or(std::path::Path::new("."))),
        file,
    );
    json!({ "report": report, "file": file, "editable": editable, "pending": pending }).to_string()
}

fn read_body(request: &mut Request) -> Result<String, String> {
    let mut body = String::new();
    let read = request
        .as_reader()
        .take(MAX_REQUEST)
        .read_to_string(&mut body);
    read.map(|_| body)
        .map_err(|_| "unreadable request body".to_string())
}

pub(super) fn save(path: &std::path::Path, body: &str) -> Result<BomFile, (u16, String)> {
    let file: BomFile = serde_json::from_str(body).map_err(|e| (400, format!("bad BOM: {e}")))?;
    file.save(path).map_err(|e| match e {
        BomError::Invalid(message) => (400, message),
        other => (500, other.to_string()),
    })?;
    Ok(file)
}

fn update(mut request: Request, state: &State, port: u16) {
    if forbidden(&request, port) {
        return respond_json(request, 403, error_body("forbidden"));
    }
    let Some((manifest, _)) = FusibleConfig::nearest() else {
        return respond_json(
            request,
            409,
            error_body("no fusible.yml: nowhere to save bom.yml"),
        );
    };
    let body = match read_body(&mut request) {
        Ok(body) => body,
        Err(message) => return respond_json(request, 400, error_body(&message)),
    };
    match save(&bom::bom_path(&manifest), &body) {
        Ok(file) => respond_json(request, 200, reply(state, &file, true)),
        Err((status, message)) => respond_json(request, status, error_body(&message)),
    }
}

fn read(request: Request, state: &State) {
    let Some((manifest, _)) = FusibleConfig::nearest() else {
        return respond_json(request, 200, reply(state, &BomFile::default(), false));
    };
    match BomFile::load_or_create(&bom::bom_path(&manifest)) {
        Ok(file) => respond_json(request, 200, reply(state, &file, true)),
        Err(err) => respond_json(request, 422, error_body(&err.to_string())),
    }
}

/// Serves the report on GET and saves the edited file on POST.
pub(super) fn serve(request: Request, state: &State, port: u16) {
    match request.method() {
        Method::Get => read(request, state),
        Method::Post => update(request, state, port),
        _ => respond_json(request, 405, error_body("use GET or POST")),
    }
}
