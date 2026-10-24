//! Errors of the BOM file and its detection.

use miette::Diagnostic;
use thiserror::Error;

/// A BOM operation that could not finish.
#[derive(Debug, Error, Diagnostic)]
pub enum BomError {
    #[error("cannot access \"{path}\"")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("cannot parse \"{path}\": {message}")]
    #[diagnostic(help("check the YAML syntax and the field names of bom.yml"))]
    Parse { path: String, message: String },

    #[error("invalid BOM: {0}")]
    Invalid(String),

    #[error("the Circuit JSON is not an array of elements: {0}")]
    Circuit(String),
}
