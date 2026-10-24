//! KiCad project to Circuit JSON conversion utilities.

use std::path::{Path, PathBuf};

/// Converts a KiCad file or directory into Circuit JSON using `kicad-to-circuit-json`.
pub fn convert_kicad(entry: &Path) -> Result<(PathBuf, String), String> {
    let mut file_paths = Vec::new();
    collect_kicad_files(entry, &mut file_paths);
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let dist_dir = cwd.join("dist");
    std::fs::create_dir_all(&dist_dir).map_err(|e| e.to_string())?;
    let out_json_path = dist_dir.join("circuit.json");

    println!(
        "📐 Converting KiCad design (\"{}\") to Circuit JSON...",
        entry.display()
    );

    let abs_paths: Vec<PathBuf> = file_paths.iter().map(|p| cwd.join(p)).collect();
    let script = conversion_script(&abs_paths, &out_json_path)?;

    let started = std::time::SystemTime::now();
    let node_path = assemble_node_path()?;
    run_script_with_runners(&script, &node_path)?;
    let fresh = std::fs::metadata(&out_json_path)
        .and_then(|m| m.modified())
        .is_ok_and(|t| t >= started);
    if !fresh {
        return Err(format!(
            "kicad-to-circuit-json did not write {}",
            out_json_path.display()
        ));
    }
    let content = std::fs::read_to_string(&out_json_path).map_err(|e| e.to_string())?;
    Ok((out_json_path, content))
}

/// Renders the JavaScript that feeds `abs_paths` to the converter and writes `out_json_path`.
fn conversion_script(abs_paths: &[PathBuf], out_json_path: &Path) -> Result<String, String> {
    let paths = serde_json::to_string(abs_paths).map_err(|e| e.to_string())?;
    let out = serde_json::to_string(out_json_path).map_err(|e| e.to_string())?;
    Ok(format!(
        r#"
const {{ KicadToCircuitJsonConverter }} = require("kicad-to-circuit-json");
const fs = require("fs");
const path = require("path");
const converter = new KicadToCircuitJsonConverter();
for (const p of {paths}) {{
    converter.addFile(path.basename(p), fs.readFileSync(p, "utf-8"));
}}
converter.runUntilFinished();
fs.writeFileSync({out}, JSON.stringify(converter.getOutput(), null, 2));
"#,
    ))
}

/// Collects the KiCad schematic and PCB files designated by `entry`.
fn collect_kicad_files(entry: &Path, file_paths: &mut Vec<PathBuf>) {
    if entry.is_file() {
        file_paths.push(entry.to_path_buf());
        file_paths.extend(sibling_kicad_files(entry));
    } else if entry.is_dir() {
        file_paths.extend(kicad_files_in_dir(entry));
    }
}

/// Returns the existing `.kicad_pcb` and `.kicad_sch` files sharing `file`'s stem.
fn sibling_kicad_files(file: &Path) -> Vec<PathBuf> {
    let (Some(stem), Some(parent)) = (file.file_stem().and_then(|s| s.to_str()), file.parent())
    else {
        return Vec::new();
    };
    ["kicad_pcb", "kicad_sch"]
        .iter()
        .map(|ext| parent.join(format!("{stem}.{ext}")))
        .filter(|p| p.is_file())
        .collect()
}

/// Returns every `.kicad_sch` and `.kicad_pcb` file directly inside `dir`.
fn kicad_files_in_dir(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            matches!(
                p.extension().and_then(|x| x.to_str()),
                Some("kicad_sch") | Some("kicad_pcb")
            )
        })
        .collect()
}

/// Builds a `NODE_PATH` that also covers local and global package installs.
fn assemble_node_path() -> Result<std::ffi::OsString, String> {
    let mut node_paths: Vec<PathBuf> = std::env::var_os("NODE_PATH")
        .map(|np| std::env::split_paths(&np).collect())
        .unwrap_or_default();
    node_paths.push(PathBuf::from("node_modules"));
    node_paths.push(PathBuf::from("../node_modules"));
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let home = PathBuf::from(home);
        node_paths.push(home.join(".bun/install/global/node_modules"));
        node_paths.push(home.join(".npm-global/lib/node_modules"));
        node_paths.push(home.join("node_modules"));
    }
    std::env::join_paths(node_paths).map_err(|e| e.to_string())
}

/// Runs `script` with bun, falling back to node.
fn run_script_with_runners(script: &str, node_path: &std::ffi::OsStr) -> Result<(), String> {
    let mut last_error = "neither bun nor node is installed".to_string();
    let bun = super::bun_path().unwrap_or_else(|| PathBuf::from("bun"));
    for runner_bin in [bun, PathBuf::from("node")] {
        let out = std::process::Command::new(&runner_bin)
            .arg("-e")
            .arg(script)
            .env("NODE_PATH", node_path)
            .output();
        match out {
            Ok(out) if out.status.success() => return Ok(()),
            Ok(out) => {
                last_error = format!(
                    "{} could not run kicad-to-circuit-json: {}",
                    runner_bin.display(),
                    String::from_utf8_lossy(&out.stderr).trim()
                )
            }
            Err(_) => continue,
        }
    }
    Err(last_error)
}
