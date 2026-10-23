//! Converts an authored STEP or STL file into a GLB with the embedded occt-wasm script.

use std::path::Path;

use serde::Serialize;

use super::generator::run_script;
use super::BodyError;

#[derive(Serialize)]
struct Job<'a> {
    input: &'a Path,
    output: &'a Path,
    format: &'a str,
}

/// Writes `output` as the GLB of `input`, whose `format` is `step` or `stl`.
pub(crate) fn convert_to_glb(input: &Path, format: &str, output: &Path) -> Result<(), BodyError> {
    println!("🧊 Converting {} to GLB...", input.display());
    let job = Job {
        input,
        output,
        format,
    };
    let payload = serde_json::to_vec(&job).map_err(std::io::Error::other)?;
    run_script("convert.ts", &payload)
}
