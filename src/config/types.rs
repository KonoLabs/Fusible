//! Named manifest sections: boards, firmware, and bodies.

use std::path::PathBuf;

use serde::Deserialize;

/// A circuit board: its tscircuit entry, output directory, dev port, and target silicon.
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BoardConfig {
    #[serde(default)]
    pub entry: Option<PathBuf>,
    #[serde(default, alias = "out-dir")]
    pub out_dir: Option<PathBuf>,
    #[serde(default, alias = "circuit-json")]
    pub circuit_json: Option<PathBuf>,
    #[serde(default)]
    pub port: Option<u16>,
    /// Target silicon identifier, e.g. `stm32g431` or `esp32-s3`.
    #[serde(default)]
    pub mcu: Option<String>,
}

/// A firmware crate bound to one board, with its generated pin bindings and build command.
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FirmwareConfig {
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub framework: Option<String>,
    #[serde(default, alias = "pins-rs")]
    pub pins_rs: Option<PathBuf>,
    #[serde(default, alias = "build-cmd")]
    pub build_cmd: Option<String>,
    /// Board the firmware runs on; optional when the project has a single board.
    #[serde(default)]
    pub board: Option<String>,
}

/// A physical body: either a file authored in any CAD tool or a model Fusible generates.
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct BodyConfig {
    /// Authored model file (STEP, STL, ...). Fusible reads it and never overwrites it.
    #[serde(default)]
    pub source: Option<PathBuf>,
    /// Command that produces `source`, for code-CAD tools such as build123d or OpenSCAD.
    #[serde(default)]
    pub build: Option<String>,
    /// Generator that owns this body, regenerated on every run: `enclosure`.
    #[serde(default)]
    pub generate: Option<String>,
    /// Board the body belongs to. Required for a generated body when the project has several
    /// boards. An authored body is modeled in this board's coordinate frame.
    #[serde(default, alias = "around")]
    pub board: Option<String>,
    #[serde(flatten)]
    pub enclosure: EnclosureConfig,
}

/// Settings of the `enclosure` generator (base + lid around a board outline).
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct EnclosureConfig {
    /// Export format: `step`, `stl`, `glb`, `brep`, or `all`.
    #[serde(default)]
    pub format: Option<String>,
    /// Directory receiving the generated models.
    #[serde(default, alias = "out-dir")]
    pub out_dir: Option<PathBuf>,
    /// Wall, floor, and lid thickness in millimetres.
    #[serde(default, alias = "wall-thickness")]
    pub wall_thickness: Option<f64>,
    /// Total external height of the base in millimetres.
    #[serde(default)]
    pub height: Option<f64>,
    /// Cut wall notches for components that reach the board edge.
    #[serde(default)]
    pub cutouts: Option<bool>,
    /// Lid ventilation pattern: `none`, `grid`, or `slots`.
    #[serde(default)]
    pub ventilation: Option<String>,
    /// Add screw bosses under every `pcb_hole` of the board.
    #[serde(default)]
    pub mounting: Option<bool>,
}
