//! Step 1 — command-line definition.
//!
//! clap handles CLI arguments and subcommands (`dev`, `export`, `build`, `check`, `init`).

use std::path::PathBuf;

use clap::builder::ValueHint;
use clap::{Parser, Subcommand};

/// Fusible — electronic linter & hardware-firmware bridge for tscircuit.
#[derive(Debug, Parser)]
#[command(
    name = "fusible",
    version,
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
#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum Commands {
    /// Start the interactive development server and web viewer
    #[command(alias = "view", alias = "webview")]
    Dev {
        /// Entry file (.tsx) or directory
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Port to run dev server on (default: 3020)
        #[arg(short = 'p', long)]
        port: Option<u16>,
    },

    /// Run the electronic linter on a circuit file or monorepo project
    #[command(alias = "lint")]
    Check {
        /// Path to Circuit JSON or TSX entry file
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Silicon target, e.g. `stm32g431`
        #[arg(long, value_name = "MCU")]
        mcu: Option<String>,
    },

    /// Compile circuit artifacts and build firmware
    Build {
        /// Path to entry TSX file
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },

    /// Export tscircuit code to various formats (gerbers, svg, glb, etc.)
    Export {
        /// Path to entry TSX file
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Export preset batch: fab (gerbers + bom + pnp), cad (kicad + step), docs (svg + pdf), or all
        #[arg(long, value_name = "PRESET")]
        preset: Option<String>,

        /// Output format (gerbers, pcb-svg, schematic-svg, glb, step, kicad_zip, etc.)
        #[arg(short = 'f', long, default_value = "gerbers")]
        format: String,

        /// Output destination path or directory
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Skip electrical rule verification before exporting
        #[arg(long)]
        skip_lint: bool,
    },

    /// Initialize a new fusible.yml project manifest in the current directory
    Init {
        /// Optional project name
        #[arg(long)]
        name: Option<String>,
    },

    /// Inspect toolchain health and available tscircuit runners
    #[command(alias = "setup")]
    Doctor {
        /// Automatically install tscircuit runner if missing
        #[arg(long)]
        install: bool,
    },
}
