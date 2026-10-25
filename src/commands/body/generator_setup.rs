//! Writes the embedded generator scripts into the cache directory and installs occt-wasm.

use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Mutex, PoisonError};

use super::BodyError;

const GENERATOR_TS: &str = include_str!("../../assets/enclosure.ts");
const CONVERT_TS: &str = include_str!("../../assets/convert.ts");
const ASSEMBLY_TS: &str = include_str!("../../assets/assembly.ts");
const PROBES_TS: &str = include_str!("../../assets/probes.ts");
pub(super) const OCCT_WASM_VERSION: &str = "5.7.1";
static PREPARE: Mutex<()> = Mutex::new(());

fn package_json() -> String {
    format!(
        "{{\"name\":\"fusible-body\",\"private\":true,\"type\":\"module\",\
\"dependencies\":{{\"occt-wasm\":\"{OCCT_WASM_VERSION}\"}}}}\n"
    )
}

/// Writes `content` to `path` through a temporary file, so a running script never reads half of it.
fn write_atomic(path: &Path, content: &str) -> std::io::Result<()> {
    if std::fs::read_to_string(path).is_ok_and(|current| current == content) {
        return Ok(());
    }
    let partial = path.with_extension(format!("{}.tmp", std::process::id()));
    std::fs::write(&partial, content)?;
    std::fs::rename(&partial, path)
}

/// Writes the scripts into `dir` and installs occt-wasm once; callers in one process take turns.
pub(super) fn prepare_generator(bun: &Path, dir: &Path) -> Result<(), BodyError> {
    let _turn = PREPARE.lock().unwrap_or_else(PoisonError::into_inner);
    std::fs::create_dir_all(dir)?;
    write_atomic(&dir.join("generator.ts"), GENERATOR_TS)?;
    write_atomic(&dir.join("convert.ts"), CONVERT_TS)?;
    write_atomic(&dir.join("assembly.ts"), ASSEMBLY_TS)?;
    write_atomic(&dir.join("probes.ts"), PROBES_TS)?;
    write_atomic(&dir.join("package.json"), &package_json())?;
    if dir.join("node_modules/occt-wasm/package.json").exists() {
        return Ok(());
    }
    println!("📦 Installing occt-wasm {OCCT_WASM_VERSION} (first run only)...");
    let status = Command::new(bun)
        .arg("install")
        .current_dir(dir)
        .stdin(Stdio::null())
        .status()?;
    if status.success() {
        return Ok(());
    }
    Err(BodyError::GeneratorFailed {
        command: "bun install".into(),
        code: status.code(),
        stderr: format!("could not install occt-wasm into {}", dir.display()),
    })
}
