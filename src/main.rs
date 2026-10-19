//! Fusible — phase 1 (MVP).
//!
//! Flow: read the file → parse → lint → render diagnostics via miette, or
//! print `✅ Circuit validated (fuse intact)` when there is nothing to
//! report.
//!
//! Exit codes are part of the contract (they let CI distinguish *what*
//! failed):
//!
//! * `0` — fuse intact, circuit validated;
//! * `1` — electrical violations: the fuse blew, generation is blocked;
//! * `2` — tool error (unreadable/malformed input, bad CLI usage — clap
//!   already exits 2 for usage errors).
//!
//! The remaining plan modules (`mcu::`, `codegen::`) are deliberately
//! absent: they activate only after this phase is validated, and the fuse
//! must blow *before* any code generation happens.
mod cli;
mod linter;
mod parser;

use clap::Parser;
use thiserror::Error;

use crate::cli::Cli;
use crate::linter::{FuseBlown, Linter};

/// Exit code: electrical violations (the fuse blew).
const EXIT_VIOLATIONS: u8 = 1;
/// Exit code: the tool itself could not do its job (aligns with clap's 2).
const EXIT_TOOL_ERROR: u8 = 2;

/// The input file could not be read: rendered as a proper diagnostic rather
/// than a bare `unwrap()` backtrace.
#[derive(Debug, Error, miette::Diagnostic)]
#[error("cannot read the Circuit JSON at \"{path}\"")]
#[diagnostic(help("check that --input points to an existing, readable file"))]
struct ReadError {
    path: String,
    #[source]
    source: std::io::Error,
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();

    // 1. Read the file in one string: it doubles as the `source_code` of
    //    every miette diagnostic (the parser needs the text to place its
    //    labels, like a compiler pointing at the faulty line).
    let source = match std::fs::read_to_string(&cli.input) {
        Ok(source) => source,
        Err(source) => {
            let error = ReadError {
                path: cli.input.display().to_string(),
                source,
            };
            return fail(error, EXIT_TOOL_ERROR);
        }
    };

    // 2. Parsing: a failure already renders a rustc-style diagnostic.
    let circuit = match parser::parse(&source, &cli.input) {
        Ok(circuit) => circuit,
        Err(diagnostic) => return fail(diagnostic, EXIT_TOOL_ERROR),
    };

    // 3. Linter: every pass, every diagnostic.
    let violations = Linter::default().run(&circuit);

    // 4. Verdict of the fuse.
    if violations.is_empty() {
        println!("✅ Circuit validated (fuse intact)");
        if let Some(mcu) = cli.mcu.as_deref() {
            println!("   silicon target: {mcu} (pin-muxing: phase 2)");
        }
        return std::process::ExitCode::SUCCESS;
    }

    // The fuse blew: miette renders the aggregate error (each violation as a
    // related error) and the process exits 1 — phase 2 will refuse to
    // generate pins.rs on a non-zero exit.
    fail(FuseBlown::new(violations), EXIT_VIOLATIONS)
}

/// Renders an error through miette's fancy handler on stderr, then returns
/// the mapped exit code. The `Error: ` prefix matches what std's
/// `Termination` would print for a `Result`-returning `main`.
fn fail(
    error: impl miette::Diagnostic + Send + Sync + 'static,
    code: u8,
) -> std::process::ExitCode {
    eprintln!("Error: {:?}", miette::Report::new(error));
    std::process::ExitCode::from(code)
}
