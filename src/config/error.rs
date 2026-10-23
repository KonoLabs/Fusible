//! Error types for Fusible configuration discovery and parsing.

use std::path::{Path, PathBuf};

use miette::{NamedSource, SourceSpan};
use thiserror::Error;

/// Errors that can occur when discovering, reading, or parsing a `fusible.yml` configuration.
#[derive(Debug, Error, miette::Diagnostic)]
pub enum ConfigError {
    /// Failed to read the configuration file from disk.
    #[error("cannot read configuration file at \"{}\"", path.display())]
    #[diagnostic(
        code(fusible::config::io),
        help("check that the file exists and is readable")
    )]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse the YAML configuration.
    #[error("failed to parse configuration: {message}")]
    #[diagnostic(
        code(fusible::config::parse),
        help("verify YAML syntax in your fusible configuration")
    )]
    Parse {
        message: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("parse error here")]
        span: SourceSpan,
    },

    /// The manifest parsed, but its sections contradict each other.
    #[error("invalid configuration: {0}")]
    #[diagnostic(
        code(fusible::config::invalid),
        help("check the `boards`, `firmware`, and `bodies` sections of fusible.yml")
    )]
    Invalid(String),
}

impl ConfigError {
    pub(crate) fn io(path: &Path, source: std::io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            source,
        }
    }

    pub(crate) fn from_yaml(err: serde_yaml::Error, source: &str, file_name: &str) -> Self {
        let offset = err
            .location()
            .map(|loc| loc.index())
            .unwrap_or(0)
            .min(source.len());
        let len = if offset < source.len() { 1 } else { 0 };

        Self::Parse {
            message: err.to_string(),
            src: NamedSource::new(file_name, source.to_string()),
            span: SourceSpan::new(offset.into(), len),
        }
    }
}
