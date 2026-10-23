//! `fusible body eject`: turn a generated body into a file you own.

use std::path::{Path, PathBuf};

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
/// fuse-gated unless `skip_lint` is set. A manifest edit that cannot be done safely is
/// reported with the lines to change, and the command still succeeds.
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
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let outputs = vec![Output {
        format,
        path: std::path::absolute(&path)?,
    }];
    generate(&json, &settings, outputs)?;
    println!("✅ Ejected body {name} ({format}) -> {}", path.display());
    update_manifest(&manifest, &name, &manifest_path(&path, root));
    Ok(())
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

fn update_manifest(manifest: &Path, name: &str, source: &str) {
    let edited = std::fs::read_to_string(manifest)
        .map_err(|e| e.to_string())
        .and_then(|old| eject_manifest(&old, name, source))
        .and_then(|new| std::fs::write(manifest, new).map_err(|e| e.to_string()));
    match edited {
        Ok(()) => println!(
            "✅ {}: body {name} now has `source: {source}`",
            manifest.display()
        ),
        Err(reason) => {
            println!("⚠️  Left {} unchanged: {reason}.", manifest.display());
            println!(
                "   The model is written. In body `{name}`, remove `generate:` and its settings"
            );
            println!(
                "   (format, out_dir, wall_thickness, height, cutouts, ventilation, mounting)"
            );
            println!("   and add `source: {source}`.");
        }
    }
}
