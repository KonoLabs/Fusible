//! Argument groups for subcommands with many options.

use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args, PartialEq)]
pub struct ExportArgs {
    /// Path to entry TSX file
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Export preset batch: fab (gerbers + bom + pnp), cad (kicad + step), docs (svg + pdf), or all
    #[arg(long, value_name = "PRESET")]
    pub preset: Option<String>,

    /// Output format (gerbers, pcb-svg, schematic-svg, glb, step, kicad_zip, etc.)
    #[arg(short = 'f', long, default_value = "gerbers")]
    pub format: String,

    /// Output destination path or directory
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Skip electrical rule verification before exporting
    #[arg(long)]
    pub skip_lint: bool,
}

#[derive(Debug, Args, PartialEq)]
pub struct BodyArgs {
    /// Body name, board name, or circuit file [default: every body of the manifest]
    #[arg(value_name = "TARGET")]
    pub target: Option<PathBuf>,

    /// Output format: step, stl, glb, brep, or all [default: step]
    #[arg(short = 'f', long)]
    pub format: Option<String>,

    /// Output directory [default: dist/bodies]
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Wall, floor, and lid thickness in mm [default: 2.0]
    #[arg(long)]
    pub wall_thickness: Option<f64>,

    /// Total external base height in mm [default: derived from the board stack]
    #[arg(long)]
    pub height: Option<f64>,

    /// Lid ventilation pattern: none, grid, or slots [default: none]
    #[arg(long)]
    pub ventilation: Option<String>,

    /// Do not notch walls where components reach the board edge
    #[arg(long)]
    pub no_cutouts: bool,

    /// Do not add screw bosses under the board's mounting holes
    #[arg(long)]
    pub no_mounting: bool,

    /// Skip electrical rule verification before generating the model
    #[arg(long)]
    pub skip_lint: bool,
}

#[derive(Debug, Args, PartialEq)]
pub struct RouteArgs {
    /// Path to Circuit JSON or TSX file
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Destination path for routed output
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Clearance distance in millimeters
    #[arg(long, default_value = "0.2")]
    pub clearance: f64,

    /// Trace width in millimeters
    #[arg(long, default_value = "0.25")]
    pub trace_width: f64,

    /// Number of copper layers available for routing
    #[arg(long, default_value = "2")]
    pub layers: u8,

    /// Validate routability without writing changes to disk
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args, PartialEq)]
pub struct SyncArgs {
    /// Path to KiCad PCB file (.kicad_pcb)
    #[arg(value_name = "KICAD_PCB", required = true)]
    pub kicad_pcb: PathBuf,

    /// Path to Circuit JSON or TSX entry file
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Target manual-edits.json file path
    #[arg(long, value_name = "TARGET")]
    pub target: Option<PathBuf>,

    /// Write layout changes to target file
    #[arg(long)]
    pub write: bool,

    /// Run without modifying manual-edits.json
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args, PartialEq)]
pub struct InitArgs {
    /// Target directory to initialize in (defaults to current directory)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Optional project name
    #[arg(long)]
    pub name: Option<String>,

    /// Target microcontroller (e.g. esp32-s3, stm32g431, rp2040)
    #[arg(long)]
    pub mcu: Option<String>,

    /// Skip installing hardware npm/bun dependencies
    #[arg(long)]
    pub no_install: bool,
}
