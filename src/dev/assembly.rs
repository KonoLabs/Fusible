//! `GET /api/assembly` and `GET /api/assembly/model`: the diagnostic preview of `fusible assembly`.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde_json::{json, Value};
use tiny_http::Request;

use super::api::{error_body, JSON};
use super::assets::http_response;
use super::body::no_store;
use crate::config::FusibleConfig;

const GLB: &str = "model/gltf-binary";

/// Whether `path` is one of the assembly routes.
pub(super) fn owns(path: &str) -> bool {
    matches!(path, "/api/assembly" | "/api/assembly/model")
}

fn preview_dir(cfg: &FusibleConfig, root: &Path) -> Option<PathBuf> {
    let assembly = cfg.assembly.as_ref()?;
    let out_dir = assembly.out_dir.clone().unwrap_or_else(|| "dist".into());
    Some(root.join(out_dir).join("assembly"))
}

fn revision(path: &Path) -> Option<u128> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    Some(modified.duration_since(UNIX_EPOCH).ok()?.as_millis())
}

/// The JSON of `GET /api/assembly` for the project rooted at `root`.
pub(super) fn report_json(cfg: Option<&FusibleConfig>, root: &Path) -> String {
    let Some(dir) = cfg.and_then(|c| preview_dir(c, root)) else {
        return json!({ "declared": false, "report": null, "model": null }).to_string();
    };
    let report: Option<Value> = std::fs::read_to_string(dir.join("check.json"))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok());
    let model =
        revision(&dir.join("preview.glb")).map(|rev| format!("/api/assembly/model?rev={rev}"));
    json!({ "declared": true, "report": report, "model": model }).to_string()
}

/// The status, content type and body of `GET /api/assembly/model`: the preview GLB, or a JSON error.
pub(super) fn model_reply(
    cfg: Option<&FusibleConfig>,
    root: &Path,
) -> (u16, &'static str, Vec<u8>) {
    let missing = |message: &str| (404, JSON, error_body(message).into_bytes());
    let Some(dir) = cfg.and_then(|c| preview_dir(c, root)) else {
        return missing("no assembly declared");
    };
    match std::fs::read(dir.join("preview.glb")) {
        Ok(glb) => (200, GLB, glb),
        Err(_) => missing("no assembly preview; run fusible assembly"),
    }
}

/// Answers a request for one of the assembly routes.
pub(super) fn serve(request: Request, path: &str) {
    let (manifest, cfg) = FusibleConfig::nearest().unzip();
    let root = manifest
        .as_deref()
        .and_then(Path::parent)
        .unwrap_or(Path::new(""));
    let response = if path == "/api/assembly" {
        http_response(200, JSON, report_json(cfg.as_ref(), root).into_bytes())
    } else {
        let (status, content_type, body) = model_reply(cfg.as_ref(), root);
        http_response(status, content_type, body)
    };
    let _ = request.respond(no_store(response));
}
