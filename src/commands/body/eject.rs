//! `fusible body eject`: turn a generated body into a file you own.

use std::path::{Path, PathBuf};

use super::eject_files::{publish_new, staging_path, write_atomic};
use super::eject_manifest::eject_manifest;
use super::settings::{resolve_settings, Settings};
use super::spec::Output;
use super::{fuse_gate, generate, BodyError, FORMATS};
use crate::commands::target::resolve_board;
use crate::config::{EnclosureConfig, FusibleConfig};

/// Generates body `name` once into a file, then rewrites its manifest entry to `source:`.
///
/// The file goes to `bodies/<name>.<format>` next to the manifest unless `output` is given;
/// its extension picks the format. An existing file is never overwritten. Generation is
/// fuse-gated unless `skip_lint` is set. The model is staged next to its final path and
/// moved into place only when generation succeeds, so a failed run leaves nothing behind.
/// A manifest edit that cannot be done safely leaves `fusible.yml` untouched and fails with
/// a tool error (exit 2) that lists the lines to change by hand.
pub fn eject(name: String, output: Option<PathBuf>, skip_lint: bool) -> Result<(), BodyError> {
    let (manifest, cfg) = FusibleConfig::nearest().ok_or_else(|| unknown(&name, None))?;
    let body = cfg
        .bodies
        .get(&name)
        .ok_or_else(|| unknown(&name, Some(&cfg)))?;
    if body.generate.is_none() {
        return Err(BodyError::NotGenerated { name });
    }
    let settings = resolve_settings(EnclosureConfig::default(), body.enclosure.clone())?;
    let root = manifest
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let (format, path) = destination(&settings, output, &name, root)?;
    if path.exists() {
        return Err(BodyError::Exists { path });
    }
    let board = resolve_board(body.board.clone().map(PathBuf::from))?;
    let json = fuse_gate(&board, skip_lint)?;
    generate_staged(&json, &settings, format, &path)?;
    println!("✅ Ejected body {name} ({format}) -> {}", path.display());
    update_manifest(&manifest, &name, &manifest_path(&path, root))
}

/// Generates the model next to `path` and moves it into place; a failure leaves no file behind.
fn generate_staged(
    json: &str,
    settings: &Settings,
    format: &'static str,
    path: &Path,
) -> Result<(), BodyError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let staged = staging_path(path);
    let outputs = vec![Output {
        format,
        path: std::path::absolute(&staged)?,
    }];
    if let Err(err) = generate(json, settings, outputs) {
        let _ = std::fs::remove_file(&staged);
        return Err(err);
    }
    publish_new(&staged, path)
}

fn unknown(name: &str, cfg: Option<&FusibleConfig>) -> BodyError {
    let generated: Vec<&str> = cfg.map_or_else(Vec::new, |c| {
        let bodies = c.bodies.iter().filter(|(_, b)| b.generate.is_some());
        bodies.map(|(n, _)| n.as_str()).collect()
    });
    let help = match generated.is_empty() {
        true => "this project has no generated bodies".to_string(),
        false => format!("generated bodies: {}", generated.join(", ")),
    };
    BodyError::UnknownBody {
        name: name.to_string(),
        help,
    }
}

/// The format and file for the ejected model: from `output`'s extension, else the first configured format.
fn destination(
    settings: &Settings,
    output: Option<PathBuf>,
    name: &str,
    root: &Path,
) -> Result<(&'static str, PathBuf), BodyError> {
    let Some(path) = output else {
        let format = settings.formats.first().copied().unwrap_or("step");
        return Ok((format, root.join("bodies").join(format!("{name}.{format}"))));
    };
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    FORMATS
        .iter()
        .find(|&&f| f == ext)
        .map(|&f| (f, path.clone()))
        .ok_or_else(|| BodyError::UnknownFormat {
            format: ext.to_string(),
        })
}

/// The path as written in the manifest: relative to its directory when the file lives below it.
fn manifest_path(path: &Path, root: &Path) -> String {
    let relative = std::path::absolute(path)
        .ok()
        .zip(std::path::absolute(root).ok())
        .and_then(|(file, dir)| file.strip_prefix(dir).map(Path::to_path_buf).ok());
    relative
        .unwrap_or_else(|| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
}

pub(super) fn update_manifest(manifest: &Path, name: &str, source: &str) -> Result<(), BodyError> {
    match edit_manifest(manifest, name, source) {
        Ok(()) => {
            println!(
                "✅ {}: body {name} now has `source: {source}`",
                manifest.display()
            );
            Ok(())
        }
        Err(reason) => Err(BodyError::ManifestUnchanged {
            manifest: manifest.to_path_buf(),
            name: name.to_string(),
            model: source.to_string(),
            reason,
        }),
    }
}

fn edit_manifest(manifest: &Path, name: &str, source: &str) -> Result<(), String> {
    let old = std::fs::read_to_string(manifest).map_err(|e| e.to_string())?;
    let new = eject_manifest(&old, name, source)?;
    write_atomic(manifest, &new).map_err(|e| e.to_string())
}
