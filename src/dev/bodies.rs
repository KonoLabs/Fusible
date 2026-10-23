//! `GET /api/bodies/<name>.glb`: the GLB of an authored body.

use std::path::Path;

use miette::Diagnostic;
use tiny_http::Request;

use super::assets::http_response;
use super::authored::{authored_glb, Failure};
use super::body::no_store;
use super::server::percent_decode;
use crate::commands::body::{convert_to_glb, generator_dir};
use crate::config::FusibleConfig;

const PREFIX: &str = "/api/bodies/";

/// Extracts the body name from `/api/bodies/<name>.glb`.
pub(super) fn body_name(path: &str) -> Option<String> {
    percent_decode(path.strip_prefix(PREFIX)?.strip_suffix(".glb")?)
}

fn describe(error: crate::commands::body::BodyError) -> String {
    match error.help() {
        Some(help) => format!("{error}: {help}"),
        None => error.to_string(),
    }
}

/// Hides where files live: the cache directory becomes `<cache>`, the project directory `.`.
pub(super) fn scrub(message: &str, root: &Path, cache: &Path) -> String {
    let root_prefix = format!("{}{}", root.display(), std::path::MAIN_SEPARATOR);
    message
        .replace(&cache.display().to_string(), "<cache>")
        .replace(&root_prefix, "")
        .replace(&root.display().to_string(), ".")
}

fn build(path: &str) -> Result<Vec<u8>, Failure> {
    let bad = |status, message: &str| Failure::new(status, message);
    let name = body_name(path).ok_or_else(|| bad(400, "bad body name"))?;
    let (manifest, cfg) =
        FusibleConfig::nearest().ok_or_else(|| bad(404, "no fusible.yml found"))?;
    let root = manifest.parent().unwrap_or(Path::new("."));
    let cache = generator_dir();
    authored_glb(
        &cfg,
        &name,
        &cache.join("authored"),
        |input, format, output| convert_to_glb(input, format, output).map_err(describe),
    )
    .map_err(|failure| Failure::new(failure.status, scrub(&failure.message, root, &cache)))
}

/// Answers `request` for the body named in `path`, on the thread the server gave this request.
pub(super) fn serve(request: Request, path: &str) {
    let response = match build(path) {
        Ok(glb) => http_response(200, "model/gltf-binary", glb),
        Err(failure) => {
            let body = serde_json::json!({ "error": failure.message }).to_string();
            http_response(failure.status, "application/json", body.into_bytes())
        }
    };
    let _ = request.respond(no_store(response));
}
