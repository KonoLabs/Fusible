//! Loading Circuit JSON from a circuit file: TSX compilation, KiCad conversion, or plain JSON.

use std::path::{Path, PathBuf};

use crate::commands::check::{CheckError, ReadError};
use crate::commands::target::{is_kicad_target, is_tsx_target};
use crate::tscircuit::Runner;

/// Finds pre-built Circuit JSON for `entry` that is at least as new as the source file.
pub fn find_fallback_circuit_json(entry: &Path) -> Option<PathBuf> {
    let stem = entry
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();
    let parent = entry.parent().unwrap_or_else(|| Path::new(""));
    let source_time = std::fs::metadata(entry).and_then(|m| m.modified()).ok();
    let candidates = [
        parent.join("dist").join("circuit.json"),
        parent.join("dist").join(stem.as_ref()).join("circuit.json"),
        PathBuf::from("dist").join("circuit.json"),
        PathBuf::from("dist")
            .join(stem.as_ref())
            .join("circuit.json"),
        entry.with_extension("circuit.json"),
        entry.with_file_name("circuit.json"),
    ];
    candidates.into_iter().find(|p| {
        let built = std::fs::metadata(p).and_then(|m| m.modified()).ok();
        p.is_file() && built >= source_time
    })
}

fn read_json(path: PathBuf) -> Result<(PathBuf, String), CheckError> {
    let content = std::fs::read_to_string(&path).map_err(|source| ReadError {
        path: path.display().to_string(),
        source,
    })?;
    Ok((path, content))
}

fn compile_tsx(entry: &Path) -> Result<(PathBuf, String), CheckError> {
    let runner = match Runner::detect() {
        Ok(runner) => runner,
        Err(runner_err) => {
            let fallback = find_fallback_circuit_json(entry).ok_or(runner_err)?;
            eprintln!(
                "⚠️  tscircuit runner not found; using \"{}\", which is newer than the source.",
                fallback.display()
            );
            return read_json(fallback);
        }
    };
    eprintln!("⚙️  Compiling \"{}\" via tscircuit...", entry.display());
    let start = std::time::Instant::now();
    let built_json = runner.build(entry, None)?;
    eprintln!(
        "✓ Circuit compiled in {:.1}s — preview ready",
        start.elapsed().as_secs_f32()
    );
    read_json(built_json)
}

fn convert_kicad(entry: &Path) -> Result<(PathBuf, String), CheckError> {
    crate::tscircuit::convert_kicad(entry).map_err(|err_msg| {
        CheckError::Read(ReadError {
            path: entry.display().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, err_msg),
        })
    })
}

pub fn load_circuit_source(entry: &Path) -> Result<(PathBuf, String), CheckError> {
    if is_kicad_target(entry) {
        convert_kicad(entry)
    } else if is_tsx_target(entry) {
        compile_tsx(entry)
    } else {
        let content = std::fs::read_to_string(entry).map_err(|source| ReadError {
            path: entry.display().to_string(),
            source,
        })?;
        Ok((entry.to_path_buf(), content))
    }
}
