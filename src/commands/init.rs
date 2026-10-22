//! Subcommand: `fusible init` — scaffold a complete Fusible & tscircuit project.

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
    if !base_dir.exists() {
        std::fs::create_dir_all(&base_dir)
            .map_err(|e| InitError::new(format!("failed to create directory: {e}")))?;
    }

    let yml_path = base_dir.join("fusible.yml");
    let yaml_path = base_dir.join("fusible.yaml");
    if yml_path.exists() || yaml_path.exists() {
        return Err(InitError::new(
            "fusible.yml already exists in target directory",
        ));
    }

    let project_name = name.unwrap_or_else(|| {
        base_dir
            .canonicalize()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "sensor-node".to_string())
    });

    let mcu_chip = mcu.unwrap_or_else(|| "esp32-s3".to_string());
    let yaml_name = serde_yaml::to_string(&project_name)
        .map(|y| y.trim_end().to_string())
        .unwrap_or_default();
    let package_name =
        serde_json::to_string(&format!("{project_name}-hardware")).unwrap_or_default();

    let manifest_content = format!(
        r#"name: {yaml_name}

hardware:
  entry: hardware/index.tsx
  out_dir: dist
  port: 3020

mcu:
  chip: {mcu_chip}

firmware:
  path: apps/firmware
  framework: esp-idf
  pins_rs: apps/firmware/src/pins.rs
  build_cmd: cargo test
"#
    );
    std::fs::write(&yml_path, manifest_content)
        .map_err(|e| InitError::new(format!("failed to write fusible.yml: {e}")))?;

    let hw_dir = base_dir.join("hardware");
    std::fs::create_dir_all(&hw_dir)
        .map_err(|e| InitError::new(format!("failed to create hardware directory: {e}")))?;

    let pkg_json = format!(
        r#"{{
  "name": {package_name},
  "version": "0.1.0",
  "private": true,
  "dependencies": {{
    "@tscircuit/core": "^0.0.229",
    "tscircuit": "^0.0.229"
  }},
  "devDependencies": {{
    "@types/react": "^18.3.0"
  }}
}}
"#
    );
    write_new(&hw_dir.join("package.json"), &pkg_json)?;

    let tsconfig_json = r#"{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "jsxImportSource": "@tscircuit/core",
    "strict": true,
    "skipLibCheck": true
  },
  "include": ["**/*.ts", "**/*.tsx"]
}
"#;
    write_new(&hw_dir.join("tsconfig.json"), tsconfig_json)?;

    let index_tsx = r#"export default () => (
  <board width="40mm" height="30mm">
    {/* Microcontroller */}
    <chip
      name="U1"
      footprint="qfp32"
      pinLabels={{
        pin1: "GND",
        pin2: "VDD3V3",
        pin3: "GPIO0",
        pin4: "GPIO1",
      }}
    />

    {/* Power Input Header */}
    <chip
      name="J1"
      footprint="pinrow2"
      pinLabels={{
        pin1: "VDD3V3",
        pin2: "GND",
      }}
    />

    {/* Status LED with Current Limiting */}
    <resistor name="R_LED" resistance="1k" footprint="0603" />

    {/* Power Rails */}
    <trace from=".J1 > .VDD3V3" to="net.VDD3V3" />
    <trace from=".J1 > .GND" to="net.GND" />
    <trace from=".U1 > .VDD3V3" to="net.VDD3V3" />
    <trace from=".U1 > .GND" to="net.GND" />

    {/* Status LED Circuit */}
    <trace from=".U1 > .GPIO0" to=".R_LED > .pin1" />
    <trace from=".R_LED > .pin2" to="net.GND" />
  </board>
);
"#;
    write_new(&hw_dir.join("index.tsx"), index_tsx)?;

    let gitignore_path = base_dir.join(".gitignore");
    if !gitignore_path.exists() {
        let gitignore_content = "node_modules/\ndist/\n.tscircuit/\ntarget/\n*.kicad_prl\n*.bak\n";
        let _ = std::fs::write(gitignore_path, gitignore_content);
    }

    if !no_install {
        install_hardware_deps(&hw_dir);
    }

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

    Ok(())
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
