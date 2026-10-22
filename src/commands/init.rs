//! Subcommand: `fusible init` — scaffold a complete Fusible & tscircuit project.

mod templates;

use std::path::{Path, PathBuf};
use std::process::Command;

use miette::Diagnostic;
use thiserror::Error;

/// Error raised when initialization cannot be completed.
#[derive(Debug, Error, Diagnostic)]
#[error("cannot initialize project: {message}")]
#[diagnostic(help("ensure you have write permissions and target files do not already exist"))]
pub struct InitError {
    message: String,
}

impl InitError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

/// Scaffolds a new Fusible project and hardware workspace in the target directory.
pub fn run(
    path: Option<PathBuf>,
    name: Option<String>,
    mcu: Option<String>,
    no_install: bool,
) -> Result<(), InitError> {
    let base_dir = path.unwrap_or_else(|| PathBuf::from("."));
    prepare_base_dir(&base_dir)?;

    let project_name = name.unwrap_or_else(|| default_project_name(&base_dir));
    let mcu_chip = mcu.unwrap_or_else(|| "esp32-s3".to_string());
    std::fs::write(
        base_dir.join("fusible.yml"),
        templates::manifest(&project_name, &mcu_chip),
    )
    .map_err(|e| InitError::new(format!("failed to write fusible.yml: {e}")))?;

    let hw_dir = base_dir.join("hardware");
    scaffold_hardware(&hw_dir, &project_name)?;

    let gitignore_path = base_dir.join(".gitignore");
    if !gitignore_path.exists() {
        let _ = std::fs::write(gitignore_path, templates::GITIGNORE);
    }

    if !no_install {
        install_hardware_deps(&hw_dir);
    }

    print_summary(&project_name);
    Ok(())
}

/// Creates `base_dir` if needed and refuses to overwrite an existing manifest.
fn prepare_base_dir(base_dir: &Path) -> Result<(), InitError> {
    if !base_dir.exists() {
        std::fs::create_dir_all(base_dir)
            .map_err(|e| InitError::new(format!("failed to create directory: {e}")))?;
    }
    if base_dir.join("fusible.yml").exists() || base_dir.join("fusible.yaml").exists() {
        return Err(InitError::new(
            "fusible.yml already exists in target directory",
        ));
    }
    Ok(())
}

/// Names the project after its directory, or `sensor-node` when that fails.
fn default_project_name(base_dir: &Path) -> String {
    base_dir
        .canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "sensor-node".to_string())
}

/// Creates the hardware workspace and its starter files.
fn scaffold_hardware(hw_dir: &Path, project_name: &str) -> Result<(), InitError> {
    std::fs::create_dir_all(hw_dir)
        .map_err(|e| InitError::new(format!("failed to create hardware directory: {e}")))?;
    write_new(
        &hw_dir.join("package.json"),
        &templates::package_json(project_name),
    )?;
    write_new(&hw_dir.join("tsconfig.json"), templates::TSCONFIG_JSON)?;
    write_new(&hw_dir.join("index.tsx"), templates::INDEX_TSX)
}

/// Prints the scaffolded tree and suggested next commands.
fn print_summary(project_name: &str) {
    println!("✨ Successfully initialized Fusible project \"{project_name}\"");
    println!("  ├── fusible.yml           (Fusible configuration & fuse contract)");
    println!("  └── hardware/");
    println!("      ├── index.tsx         (tscircuit schematic + layout design)");
    println!("      ├── package.json      (tscircuit dependencies)");
    println!("      └── tsconfig.json");
    println!();
    println!("Next steps:");
    println!("  fusible dev               Start interactive hot-reloading webviewer");
    println!(
        "  fusible check             Validate electrical design rules & extract firmware pins"
    );
    println!("  fusible export --preset cad  Export KiCad project for Electrical Engineers");
    println!("  fusible add <pkg>         Install tscircuit packages or footprints");
    println!("  fusible build             Compile hardware artifacts & build firmware");
}

/// Writes `content` to `path` unless the file already exists, which it reports and keeps.
fn write_new(path: &Path, content: &str) -> Result<(), InitError> {
    if path.exists() {
        println!("↷ Keeping existing {}", path.display());
        return Ok(());
    }
    std::fs::write(path, content)
        .map_err(|e| InitError::new(format!("failed to write {}: {e}", path.display())))
}

/// Runs `install` with the first available package manager in `hw_dir`.
fn install_hardware_deps(hw_dir: &Path) {
    println!("📦 Installing hardware dependencies...");
    let runners = ["bun", "npm"];
    for runner in runners {
        let mut cmd = Command::new(runner);
        cmd.arg("install").current_dir(hw_dir);
        if let Ok(mut child) = cmd.spawn() {
            let _ = child.wait();
            return;
        }
    }
    println!("⚠️  Neither bun nor npm found; run 'fusible doctor --install' or install dependencies manually.");
}
