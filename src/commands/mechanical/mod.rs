//! Subcommand: `fusible mechanical` — generate a 3D enclosure model from the board outline.
//!
//! Fusible computes the whole layout (outline, screw bosses, edge notches, vents) from
//! Circuit JSON, then hands a JSON spec to an embedded TypeScript generator that runs the
//! OpenCascade kernel (`occt-wasm`) under bun and writes the requested CAD files.

mod generator;
mod layout;
mod settings;
mod spec;

#[cfg(test)]
mod layout_tests;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::check::{self, CheckError};
use crate::commands::target::resolve_entry;
use crate::config::{FusibleConfig, MechanicalConfig};
use crate::gerber::PcbModel;

use generator::{generator_dir, run_generator};
use settings::{preview_settings, resolve_settings, Settings};
use spec::{build_spec, preview_from, Output, Spec};

const FORMATS: &[&str] = &["step", "stl", "glb", "brep"];
const DEFAULT_WALL: f64 = 2.0;
const DEFAULT_BOARD_THICKNESS: f64 = 1.6;
const CLEARANCE: f64 = 0.5;
const STANDOFF: f64 = 3.0;
const HEADROOM: f64 = 12.0;
const BOSS_WALL: f64 = 2.0;
const PILOT_UNDERSIZE: f64 = 0.7;
const EDGE_TOLERANCE: f64 = 0.5;
const NOTCH_MARGIN: f64 = 0.5;
const VENT_INSET: f64 = 4.0;

/// Unified error enum for the mechanical pipeline.
#[derive(Debug, Error, Diagnostic)]
pub enum MechanicalError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Check(#[from] CheckError),

    #[error("failed to write mechanical files: {0}")]
    #[diagnostic(code(fusible::mechanical::io_error))]
    Io(#[from] std::io::Error),

    #[error("unknown mechanical format \"{format}\"")]
    #[diagnostic(
        code(fusible::mechanical::unknown_format),
        help("valid formats are: `step`, `stl`, `glb`, `brep`, or `all`")
    )]
    UnknownFormat { format: String },

    #[error("unknown ventilation pattern \"{pattern}\"")]
    #[diagnostic(
        code(fusible::mechanical::unknown_ventilation),
        help("valid patterns are: `none`, `grid`, or `slots`")
    )]
    UnknownVentilation { pattern: String },

    #[error("invalid {name}: {value} mm")]
    #[diagnostic(code(fusible::mechanical::invalid_dimension), help("{help}"))]
    InvalidDimension {
        name: &'static str,
        value: f64,
        help: String,
    },

    #[error("no usable pcb_board outline in the circuit")]
    #[diagnostic(
        code(fusible::mechanical::no_board),
        help("add a <board width=\"..\" height=\"..\" /> or an outline with at least 3 points")
    )]
    NoBoard,

    #[error("bun is required to run the mechanical generator")]
    #[diagnostic(
        code(fusible::mechanical::bun_missing),
        help("install bun from https://bun.sh, then rerun `fusible mechanical`")
    )]
    BunMissing,

    #[error("mechanical generator failed (`{command}`, exit code {code:?})")]
    #[diagnostic(code(fusible::mechanical::generator_failed), help("{stderr}"))]
    GeneratorFailed {
        command: String,
        code: Option<i32>,
        stderr: String,
    },
}

/// A studio preview of the mechanical model: the GLB bytes plus the frame needed to place it.
///
/// The model frame is in millimetres, Z-up, with X/Y equal to the Circuit JSON pcb
/// coordinates. The floor bottom sits at z=0, the board bottom face at z=`board_z`,
/// and the lid occupies z in [`height`, `height + wall`]. The GLB holds two named
/// nodes, `base` and `lid`. `size` is the outer bounding box: the outline bounds plus
/// `2 * (clearance + wall)` in X/Y, and `height + wall` in Z.
#[derive(Debug, Clone, PartialEq)]
pub struct MechanicalPreview {
    pub glb: Vec<u8>,
    pub board_z: f64,
    pub board_thickness: f64,
    pub height: f64,
    pub wall: f64,
    pub size: [f64; 3],
}

static PREVIEW_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generates a base and lid enclosure model from the board outline.
///
/// CLI values in `overrides` win over the `mechanical:` section of `fusible.yml`,
/// which wins over built-in defaults. The model is **fuse-gated**: electrical rules
/// run first and a blown fuse stops generation before any file is written.
pub fn run(
    file: Option<PathBuf>,
    overrides: MechanicalConfig,
    skip_lint: bool,
) -> Result<(), MechanicalError> {
    let manifest = load_manifest();
    let mechanical = manifest.as_ref().and_then(|m| m.mechanical.clone());
    let settings = resolve_settings(overrides, mechanical.unwrap_or_default())?;
    let entry = resolve_entry(file.clone()).unwrap_or_else(|_| PathBuf::from("index.tsx"));
    fuse_gate(file, skip_lint)?;

    let (_, circuit_json) = check::load_circuit_source(&entry)?;
    let stem = manifest
        .and_then(|m| m.name)
        .or_else(|| entry.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "board".to_string());
    let out_dir = std::path::absolute(&settings.out_dir)?;
    let outputs = settings
        .formats
        .iter()
        .map(|&format| Output {
            format,
            path: out_dir.join(format!("{stem}.{format}")),
        })
        .collect();
    std::fs::create_dir_all(&out_dir)?;
    for output in generate(&circuit_json, &settings, outputs)?.outputs {
        println!(
            "✅ Exported mechanical {} -> {}",
            output.format,
            output.path.display()
        );
    }
    Ok(())
}

/// Builds a GLB preview of the mechanical model for the dev studio.
///
/// Uses the `mechanical:` section of the manifest, but always renders GLB into a
/// temporary file inside the generator cache and ignores `out_dir` and `format`.
/// Unlike [`run`], the preview is not fuse-gated. See [`MechanicalPreview`] for the frame.
pub fn preview(
    circuit_json: &str,
    manifest: Option<MechanicalConfig>,
) -> Result<MechanicalPreview, MechanicalError> {
    let settings = preview_settings(manifest.unwrap_or_default())?;
    let id = PREVIEW_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = generator_dir().join(format!("preview-{}-{id}.glb", std::process::id()));
    let outputs = vec![Output {
        format: "glb",
        path: path.clone(),
    }];
    let result = generate(circuit_json, &settings, outputs)
        .and_then(|spec| Ok(preview_from(&spec, std::fs::read(&path)?)));
    let _ = std::fs::remove_file(&path);
    result
}

/// Loads the nearest `fusible.yml` manifest, if any.
pub(crate) fn load_manifest() -> Option<FusibleConfig> {
    FusibleConfig::find_and_load(Path::new("."))
        .ok()
        .flatten()
        .map(|(_, cfg)| cfg)
}

fn fuse_gate(file: Option<PathBuf>, skip_lint: bool) -> Result<(), MechanicalError> {
    if skip_lint {
        println!("⚠️  Skipping electrical rule verification (--skip-lint requested).\n");
        return Ok(());
    }
    println!("🔍 Verifying electrical rules before mechanical export (fuse check)...");
    check::run(file, None, None)?;
    println!("⚡ Fuse intact — proceeding with mechanical export.\n");
    Ok(())
}

fn generate(
    circuit_json: &str,
    settings: &Settings,
    outputs: Vec<Output>,
) -> Result<Spec, MechanicalError> {
    let spec = build_spec(&PcbModel::parse(circuit_json), settings, outputs)?;
    run_generator(&spec)?;
    Ok(spec)
}
