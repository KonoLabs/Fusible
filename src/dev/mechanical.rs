//! Lazy, revision-cached mechanical preview served to the dev studio.
//!
//! `GET /api/mechanical` returns the model frame as JSON and `GET /api/mechanical.glb`
//! the GLB bytes for the current revision. See
//! [`MechanicalPreview`](crate::commands::mechanical::MechanicalPreview) for the frame.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;

use miette::Diagnostic;
use tiny_http::{Header, Request};

use super::assets::{http_response, HttpResponse};
use crate::commands::mechanical::{load_manifest, preview, MechanicalPreview};

const JSON: &str = "application/json";
const TEXT: &str = "text/plain; charset=utf-8";
const GLB: &str = "model/gltf-binary";

type Generated = Result<Arc<MechanicalPreview>, String>;

#[derive(Default)]
struct Current {
    json: Option<Arc<str>>,
    revision: String,
}

/// Shared circuit source plus a one-entry cache of the generated preview, keyed by revision.
#[derive(Clone, Default)]
pub struct MechanicalCache {
    current: Arc<Mutex<Current>>,
    slot: Arc<Mutex<Option<(String, Generated)>>>,
}

impl MechanicalCache {
    /// Records the Circuit JSON source and revision of the latest rebuild.
    pub fn update(&self, json: Option<String>, revision: String) {
        *lock(&self.current) = Current {
            json: json.map(Arc::from),
            revision,
        };
    }

    /// Answers `request` on a spawned thread so the request loop never blocks on generation.
    pub fn serve(&self, request: Request, glb: bool) {
        let cache = self.clone();
        thread::spawn(move || {
            let _ = request.respond(cache.response(glb, generate));
        });
    }

    /// Returns the preview for the current revision, generating it at most once per revision.
    pub(super) fn resolve_with(
        &self,
        generate: impl FnOnce(&str) -> Generated,
    ) -> Option<(String, Generated)> {
        let mut slot = lock(&self.slot);
        let (json, revision) = {
            let current = lock(&self.current);
            (current.json.clone()?, current.revision.clone())
        };
        if let Some((cached, generated)) = slot.as_ref().filter(|(r, _)| *r == revision) {
            return Some((cached.clone(), generated.clone()));
        }
        let generated = generate(&json);
        *slot = Some((revision.clone(), generated.clone()));
        Some((revision, generated))
    }

    pub(super) fn response(
        &self,
        glb: bool,
        generate: impl FnOnce(&str) -> Generated,
    ) -> HttpResponse {
        let Some((revision, generated)) = self.resolve_with(generate) else {
            let body = serde_json::json!({ "error": "no circuit loaded" }).to_string();
            return no_store(http_response(503, JSON, body.into_bytes()));
        };
        no_store(match (generated, glb) {
            (Ok(p), true) => http_response(200, GLB, p.glb.clone()),
            (Ok(p), false) => http_response(200, JSON, meta_json(&revision, &p).into_bytes()),
            (Err(e), true) => http_response(422, TEXT, e.into_bytes()),
            (Err(e), false) => {
                let body = serde_json::json!({ "revision": revision, "error": e }).to_string();
                http_response(422, JSON, body.into_bytes())
            }
        })
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn generate(json: &str) -> Generated {
    let manifest = load_manifest().and_then(|cfg| cfg.mechanical);
    preview(json, manifest)
        .map(Arc::new)
        .map_err(|e| match e.help() {
            Some(help) => format!("{e}: {help}"),
            None => e.to_string(),
        })
}

/// Serializes the preview frame returned by `GET /api/mechanical`.
pub(super) fn meta_json(revision: &str, p: &MechanicalPreview) -> String {
    serde_json::json!({
        "revision": revision,
        "board_z": p.board_z,
        "board_thickness": p.board_thickness,
        "height": p.height,
        "wall": p.wall,
        "size": p.size,
    })
    .to_string()
}

fn no_store(response: HttpResponse) -> HttpResponse {
    let header = Header::from_bytes(&b"Cache-Control"[..], &b"no-store"[..]).expect("valid header");
    response.with_header(header)
}
