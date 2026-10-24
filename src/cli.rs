//! Step 1 — command-line definition.
//!
//! clap handles CLI arguments and subcommands (`dev`, `export`, `build`, `check`, `init`).

use std::path::PathBuf;

use clap::builder::ValueHint;
use clap::{Parser, Subcommand};

mod args;

pub use args::{
    AssemblyArgs, BodyAction, BodyArgs, BomArgs, BomFormat, EjectArgs, ExportArgs, InitArgs,
    KitArgs, RouteArgs, SyncArgs,
};

/// Fusible — electronic linter & hardware-firmware bridge for tscircuit.
#[derive(Debug, Parser)]
#[command(
    name = "fusible",
    version,
    arg_required_else_help = true,
    about = "Electronic linter: the fuse blows to block firmware code generation",
    after_help = "Examples:\n  fusible --input circuit.json\n  fusible check\n  fusible dev"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to the Circuit JSON or TSX file (shorthand for check)
    #[arg(short = 'i', long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub input: Option<PathBuf>,

    /// Silicon target, e.g. `stm32g431` (consumed by pin-muxing in phase 2)
    #[arg(long, value_name = "MCU")]
    pub mcu: Option<String>,
}

/// Available subcommands for managing tscircuit monorepo projects.
#[derive(Debug, Subcommand, PartialEq)]
pub enum Commands {
    /// Start the interactive development server and web viewer
    #[command(alias = "view", alias = "webview")]
    Dev {
        /// Entry file (.tsx, .json) or directory
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Port to run dev server on (default: 3020)
        #[arg(short = 'p', long)]
        port: Option<u16>,

        /// Use legacy tsci dev server instead of native Fusible Studio
        #[arg(long)]
        legacy: bool,

        /// Run under the desktop shell: print a ready line, exit when stdin closes
        #[arg(long)]
        desktop: bool,
    },

    /// Run the electronic linter on a circuit file or monorepo project
    #[command(alias = "lint")]
    Check {
        /// Path to Circuit JSON, TSX, or KiCad entry file
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Silicon target, e.g. `stm32g431` or `esp32-s3`
        #[arg(long, value_name = "MCU")]
        mcu: Option<String>,

        /// Destination path for generated firmware pin bindings (pins.rs)
        #[arg(long, value_name = "PATH")]
        pins_rs: Option<PathBuf>,
    },

    /// Compile circuit artifacts and build firmware
    Build {
        /// Path to entry TSX file
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },

    /// Export tscircuit code to various formats (gerbers, svg, glb, etc.)
    Export(ExportArgs),

    /// Build the physical bodies: authored CAD files and generated enclosures
    #[command(
        alias = "bodies",
        alias = "mechanical",
        alias = "case",
        alias = "housing"
    )]
    Body(BodyArgs),

    /// Place the boards and bodies of `assembly:` in one frame and export one model
    #[command(alias = "asm")]
    Assembly(AssemblyArgs),

    /// List the parts of the project with quantities and prices (bill of materials)
    #[command(alias = "parts")]
    Bom(BomArgs),

    /// Run the autorouter to solve circuit traces
    Route(RouteArgs),

    /// Compare PCB footprint placement between KiCad and tscircuit
    Diff {
        /// Path to KiCad PCB file (.kicad_pcb)
        #[arg(value_name = "KICAD_PCB", required = true)]
        kicad_pcb: PathBuf,

        /// Path to Circuit JSON or TSX entry file
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },

    /// Synchronize layout adjustments from KiCad back into tscircuit
    Sync(SyncArgs),

    /// Initialize a new Fusible project and tscircuit hardware workspace
    Init(InitArgs),

    /// Add a tscircuit component or registry package to the hardware project
    Add {
        /// Packages to install (e.g. @tsci/seveibar.smd-resistor or @tscircuit/core)
        #[arg(value_name = "PACKAGE", required = true)]
        packages: Vec<String>,
    },

    /// Install all hardware dependencies in the project
    Install {
        /// Target directory (defaults to hardware/ or current directory)
        #[arg(value_name = "DIR")]
        dir: Option<PathBuf>,
    },

    /// Search the tscircuit registry for components, footprints, and CAD models
    Search {
        /// Search terms (e.g. "esp32", "usb-c", "resistor 0603")
        #[arg(value_name = "QUERY", required = true)]
        query: Vec<String>,
    },

    /// Inspect toolchain health and available tscircuit runners
    #[command(alias = "setup")]
    Doctor {
        /// Automatically install tscircuit runner if missing
        #[arg(long)]
        install: bool,

        /// Print the toolchain as JSON and exit 0 even when tools are missing
        #[arg(long)]
        json: bool,
    },
}
