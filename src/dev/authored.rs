//! GLB of an authored body for `GET /api/bodies/<name>.glb`.
//!
//! Only files declared as a body `source:` in the manifest are read: the URL carries the
//! body name, never a path. GLB is served as is. STEP and STL are converted to GLB with
//! occt-wasm and cached by path, modification time, and size.

use std::io::Read;
use std::path::{Path, PathBuf};

use super::authored_cache::{cache_key, cached_conversion};
use crate::config::FusibleConfig;

const MAX_BYTES: u64 = 64 * 1024 * 1024;

/// Text that marks a conversion error as a timeout, which is reported as a 504.
pub(super) const TIMEOUT_MARK: &str = "timed out after";

/// Why a body GLB could not be served, with the HTTP status that reports it.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Failure {
    pub status: u16,
    pub message: String,
}

impl Failure {
    pub(super) fn new(status: u16, message: impl Into<String>) -> Self {
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
        Kind::Glb => read_capped(&source),
        Kind::Convert(format) => {
            let key = cache_key(&source, &meta);
            cached_conversion(&source, format, &key, cache_dir, &convert)
        }
    }
}

/// Reads `path`, refusing more than [`MAX_BYTES`] even when the file grew after the size check.
fn read_capped(path: &Path) -> Result<Vec<u8>, Failure> {
    let io = |e: std::io::Error| Failure::new(500, e.to_string());
    let mut bytes = Vec::new();
    let file = std::fs::File::open(path).map_err(io)?;
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(io)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(Failure::new(422, "source file is larger than 64 MiB"));
    }
    Ok(bytes)
}
