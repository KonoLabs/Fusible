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
    let paths = serde_json::to_string(&abs_paths).map_err(|e| e.to_string())?;
    let out = serde_json::to_string(&out_json_path).map_err(|e| e.to_string())?;
    let script = format!(
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
    );

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

fn collect_kicad_files(entry: &Path, file_paths: &mut Vec<PathBuf>) {
    if entry.is_file() {
        file_paths.push(entry.to_path_buf());
        if let Some(stem) = entry.file_stem().and_then(|s| s.to_str()) {
            if let Some(parent) = entry.parent() {
                let pcb_path = parent.join(format!("{stem}.kicad_pcb"));
                let sch_path = parent.join(format!("{stem}.kicad_sch"));
                if pcb_path.is_file() {
                    file_paths.push(pcb_path);
                }
                if sch_path.is_file() {
                    file_paths.push(sch_path);
                }
            }
        }
    } else if entry.is_dir() {
        if let Ok(entries) = std::fs::read_dir(entry) {
            for e in entries.filter_map(Result::ok) {
                let p = e.path();
                if let Some(ext) = p.extension().and_then(|x| x.to_str()) {
                    if ext == "kicad_sch" || ext == "kicad_pcb" {
                        file_paths.push(p);
                    }
                }
            }
        }
    }
}

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

fn run_script_with_runners(script: &str, node_path: &std::ffi::OsStr) -> Result<(), String> {
    let mut last_error = "neither bun nor node is installed".to_string();
    for runner_bin in ["bun", "node"] {
        let out = std::process::Command::new(runner_bin)
            .arg("-e")
            .arg(script)
            .env("NODE_PATH", node_path)
            .output();
        match out {
            Ok(out) if out.status.success() => return Ok(()),
            Ok(out) => {
                last_error = format!(
                    "{runner_bin} could not run kicad-to-circuit-json: {}",
                    String::from_utf8_lossy(&out.stderr).trim()
                )
            }
            Err(_) => continue,
        }
    }
    Err(last_error)
}
