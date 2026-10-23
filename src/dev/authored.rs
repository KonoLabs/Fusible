//! GLB of an authored body for `GET /api/bodies/<name>.glb`.
//!
//! Only files declared as a body `source:` in the manifest are read: the URL carries the
//! body name, never a path. GLB is served as is. STEP and STL are converted to GLB with
//! occt-wasm and cached by path, modification time, and size.

use std::collections::hash_map::DefaultHasher;
use std::fs::Metadata;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::config::FusibleConfig;

const MAX_BYTES: u64 = 64 * 1024 * 1024;

/// Why a body GLB could not be served, with the HTTP status that reports it.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Failure {
    pub status: u16,
    pub message: String,
}

impl Failure {
    fn new(status: u16, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

enum Kind {
    Glb,
    Convert(&'static str),
}

fn kind_of(path: &Path) -> Option<Kind> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "glb" => Some(Kind::Glb),
        "step" | "stp" => Some(Kind::Convert("step")),
        "stl" => Some(Kind::Convert("stl")),
        _ => None,
    }
}

fn source_of(cfg: &FusibleConfig, name: &str) -> Result<PathBuf, Failure> {
    let body = cfg
        .bodies
        .get(name)
        .ok_or_else(|| Failure::new(404, format!("unknown body `{name}`")))?;
    body.source.clone().ok_or_else(|| {
        let help = "generated bodies are served by /api/mechanical";
        Failure::new(404, format!("body `{name}` is not authored; {help}"))
    })
}

/// Hashes the path alone, then the path with its modification time and size.
pub(super) fn cache_key(path: &Path, meta: &Metadata) -> String {
    let nanos = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos());
    let mut by_path = DefaultHasher::new();
    path.hash(&mut by_path);
    let mut by_state = DefaultHasher::new();
    (path, nanos, meta.len()).hash(&mut by_state);
    format!("{:016x}-{:016x}", by_path.finish(), by_state.finish())
}

fn prune_stale(cache_dir: &Path, key: &str) {
    let prefix = key.split('-').next().unwrap_or(key);
    let stale = std::fs::read_dir(cache_dir).into_iter().flatten().flatten();
    for entry in stale {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(prefix) && !name.starts_with(key) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

fn cached_conversion(
    source: &Path,
    format: &str,
    key: &str,
    cache_dir: &Path,
    convert: &impl Fn(&Path, &str, &Path) -> Result<(), String>,
) -> Result<Vec<u8>, Failure> {
    let cached = cache_dir.join(format!("{key}.glb"));
    if let Ok(bytes) = std::fs::read(&cached) {
        return Ok(bytes);
    }
    let io = |e: std::io::Error| Failure::new(500, format!("cannot use the body cache: {e}"));
    std::fs::create_dir_all(cache_dir).map_err(io)?;
    let partial = cache_dir.join(format!("{key}.{}.part", std::process::id()));
    convert(source, format, &partial)
        .map_err(|e| Failure::new(422, format!("conversion failed: {e}")))?;
    std::fs::rename(&partial, &cached).map_err(io)?;
    prune_stale(cache_dir, key);
    std::fs::read(&cached).map_err(io)
}

/// Returns the GLB bytes of the authored body `name`, converting and caching when needed.
///
/// `convert(source, format, output)` writes the GLB of a STEP or STL file to `output`.
pub(super) fn authored_glb(
    cfg: &FusibleConfig,
    name: &str,
    cache_dir: &Path,
    convert: impl Fn(&Path, &str, &Path) -> Result<(), String>,
) -> Result<Vec<u8>, Failure> {
    let source = source_of(cfg, name)?;
    let meta = std::fs::metadata(&source).map_err(|_| {
        Failure::new(
            404,
            format!("source file \"{}\" does not exist", source.display()),
        )
    })?;
    if meta.len() > MAX_BYTES {
        let limit = MAX_BYTES / (1024 * 1024);
        return Err(Failure::new(
            422,
            format!("source file is larger than {limit} MiB"),
        ));
    }
    let Some(kind) = kind_of(&source) else {
        let message = "unsupported source type; use .glb, .step, .stp, or .stl";
        return Err(Failure::new(422, message));
    };
    match kind {
        Kind::Glb => std::fs::read(&source).map_err(|e| Failure::new(500, e.to_string())),
        Kind::Convert(format) => {
            let key = cache_key(&source, &meta);
            cached_conversion(&source, format, &key, cache_dir, &convert)
        }
    }
}
