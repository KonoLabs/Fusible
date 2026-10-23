//! Installs and runs the embedded bun + occt-wasm generator.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::spec::Spec;
use super::BodyError;
use crate::tscircuit::detect::find_binary_in_path;

const GENERATOR_TS: &str = include_str!("../../assets/enclosure.ts");
const CONVERT_TS: &str = include_str!("../../assets/convert.ts");
const OCCT_WASM_VERSION: &str = "5.7.1";

/// Returns the per-version cache directory that holds the generator and its dependencies.
pub(crate) fn generator_dir() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .or_else(|| std::env::var_os("LOCALAPPDATA").map(PathBuf::from))
        .unwrap_or_else(std::env::temp_dir)
        .join("fusible")
        .join(format!("body-{}", env!("CARGO_PKG_VERSION")))
}

/// Pipes `spec` to the generator, which writes every file listed in `spec.outputs`.
pub(super) fn run_generator(spec: &Spec) -> Result<(), BodyError> {
    println!("🧊 Building body model with OpenCascade...");
    let payload = serde_json::to_vec(spec).map_err(std::io::Error::other)?;
    run_script("generator.ts", &payload)
}

/// Runs the embedded script `script` under bun with `payload` on stdin; stdout is discarded.
pub(super) fn run_script(script: &str, payload: &[u8]) -> Result<(), BodyError> {
    let bun = std::env::var_os("PATH")
        .and_then(|p| find_binary_in_path("bun", &p))
        .ok_or(BodyError::BunMissing)?;
    let dir = generator_dir();
    prepare_generator(&bun, &dir)?;
    let mut child = Command::new(&bun)
        .arg(script)
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(payload)?;
    let out = child.wait_with_output()?;
    if out.status.success() {
        return Ok(());
    }
    Err(BodyError::GeneratorFailed {
        command: format!("bun {}", dir.join(script).display()),
        code: out.status.code(),
        stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
    })
}

fn package_json() -> String {
    format!(
        "{{\"name\":\"fusible-body\",\"private\":true,\"type\":\"module\",\
\"dependencies\":{{\"occt-wasm\":\"{OCCT_WASM_VERSION}\"}}}}\n"
    )
}

fn prepare_generator(bun: &Path, dir: &Path) -> Result<(), BodyError> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join("generator.ts"), GENERATOR_TS)?;
    std::fs::write(dir.join("convert.ts"), CONVERT_TS)?;
    std::fs::write(dir.join("package.json"), package_json())?;
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
