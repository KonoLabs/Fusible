//! One-off STEP export of a generated body, for callers that place it elsewhere.

use std::path::PathBuf;

use super::settings::resolve_settings;
use super::spec::Output;
use super::{generate, BodyError};
use crate::config::EnclosureConfig;

/// Writes the generated body of `circuit_json` as one STEP file at `path`, without a fuse check.
///
/// Returns the Z of the board's mid-plane in the model frame, where tscircuit's board model
/// puts Z = 0, so a caller can drop the model onto the board.
pub(crate) fn generate_step(
    circuit_json: &str,
    manifest: EnclosureConfig,
    path: PathBuf,
) -> Result<f64, BodyError> {
    let forced = EnclosureConfig {
        format: Some("step".to_string()),
        ..Default::default()
    };
    let settings = resolve_settings(forced, manifest)?;
    let outputs = vec![Output {
        format: "step",
        path,
    }];
    let spec = generate(circuit_json, &settings, outputs)?;
    Ok(spec.board_z + spec.board_thickness / 2.0)
}
