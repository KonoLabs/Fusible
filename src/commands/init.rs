//! Subcommand: `fusible init` — scaffold a starter `fusible.yml`.

use std::path::Path;

use miette::Diagnostic;
use thiserror::Error;

use crate::config::FusibleConfig;

/// Error raised when initialization cannot be completed.
#[derive(Debug, Error, Diagnostic)]
#[error("cannot initialize project: {message}")]
#[diagnostic(help("ensure you have write permissions and the file does not already exist"))]
pub struct InitError {
    message: String,
}

/// Scaffolds a new `fusible.yml` in the current directory.
pub fn run(name: Option<String>) -> Result<(), InitError> {
    let yml_path = Path::new("fusible.yml");
    let yaml_path = Path::new("fusible.yaml");
    if yml_path.exists() || yaml_path.exists() {
        return Err(InitError {
            message: "fusible.yml already exists in current directory".into(),
        });
    }

    let mut content = FusibleConfig::default_manifest().to_string();
    if let Some(name) = name {
        content = content.replace("name: sensor-node", &format!("name: {name}"));
    }

    std::fs::write(yml_path, content).map_err(|source| InitError {
        message: format!("failed to write fusible.yml: {source}"),
    })?;

    println!("✅ Created fusible.yml");
    Ok(())
}
