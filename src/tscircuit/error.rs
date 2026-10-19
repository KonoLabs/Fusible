//! Error types for tscircuit runner execution and resolution.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when discovering, invoking, or evaluating tscircuit.
#[derive(Debug, Error, miette::Diagnostic)]
pub enum RunnerError {
    /// No valid runner (local tsci, global tsci, bun, or npx) was detected.
    #[error("no tscircuit runner found")]
    #[diagnostic(
        code(fusible::tscircuit::not_found),
        help(
            "Fusible requires tscircuit to compile, export, or preview circuits.\n\
             Install bun or tscircuit using one of the following:\n\
               • bun (recommended): curl -fsSL https://bun.sh/install | bash\n\
               • global npm: npm install -g tscircuit\n\
               • local project: npm install tscircuit"
        )
    )]
    NotFound,

    /// Failed to spawn or communicate with the process.
    #[error("failed to execute `{command}`: {source}")]
    #[diagnostic(
        code(fusible::tscircuit::io_error),
        help("verify that the binary exists and has execute permissions")
    )]
    Io {
        command: String,
        #[source]
        source: std::io::Error,
    },

    /// The tscircuit command executed but exited with a failure status.
    #[error("tscircuit command `{command}` failed with exit code {code:?}")]
    #[diagnostic(
        code(fusible::tscircuit::command_failed),
        help("{help}")
    )]
    CommandFailed {
        command: String,
        code: Option<i32>,
        stderr: String,
        help: String,
    },

    /// The build completed successfully, but the expected circuit json was not found.
    #[error("circuit json output not found at \"{}\" after build", path.display())]
    #[diagnostic(
        code(fusible::tscircuit::output_not_found),
        help("verify that tscircuit build completed and generated circuit.json in the output directory")
    )]
    OutputNotFound {
        path: PathBuf,
    },
}
