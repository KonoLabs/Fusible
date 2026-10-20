//! Subcommand: `fusible search` — search the tscircuit registry for footprints and parts.

use miette::Diagnostic;
use thiserror::Error;

use crate::tscircuit::error::RunnerError;
use crate::tscircuit::Runner;

/// Error raised when registry search fails.
#[derive(Debug, Error, Diagnostic)]
pub enum SearchError {
    #[error("failed to detect tscircuit runner: {0}")]
    #[diagnostic(help("run 'fusible doctor --install' to set up a runner"))]
    Runner(#[from] RunnerError),
}

/// Searches the tscircuit component registry.
pub fn run(query: &[String]) -> Result<(), SearchError> {
    let runner = Runner::detect()?;
    runner.search(query)?;
    Ok(())
}
