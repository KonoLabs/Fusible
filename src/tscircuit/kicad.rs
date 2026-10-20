//! KiCad project to Circuit JSON conversion utilities.

use std::path::{Path, PathBuf};

/// Identifies if a path targets a KiCad file or folder.
pub fn is_kicad_target(path: &Path) -> bool {
    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for e in entries.filter_map(Result::ok) {
                if let Some(ext) = e.path().extension().and_then(|x| x.to_str()) {
                    if ext == "kicad_pro" || ext == "kicad_sch" || ext == "kicad_pcb" {
                        return true;
                    }
                }
            }
        }
    }
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("kicad_pro" | "kicad_sch" | "kicad_pcb")
    )
}

/// Converts a KiCad file or directory into Circuit JSON using `kicad-to-circuit-json`.
pub fn convert_kicad(entry: &Path) -> Result<(PathBuf, String), String> {
    let mut file_paths = Vec::new();
    collect_kicad_files(entry, &mut file_paths);

    let fallback = find_fallback_circuit_json(entry);
    let dist_dir = std::env::current_dir()
        .map(|cd| cd.join("dist"))
        .unwrap_or_else(|_| PathBuf::from("dist"));
    std::fs::create_dir_all(&dist_dir).ok();
    let out_json_path = dist_dir.join("circuit.json");

    println!(
        "📐 Converting KiCad design (\"{}\") to Circuit JSON...",
        entry.display()
    );

    let abs_paths: Vec<String> = file_paths
        .iter()
        .map(|p| {
            if p.is_absolute() {
                p.display().to_string()
            } else {
                std::env::current_dir()
                    .map(|cd| cd.join(p).display().to_string())
                    .unwrap_or_else(|_| p.display().to_string())
            }
        })
        .collect();

    let paths_json = serde_json::to_string(&abs_paths).unwrap_or_else(|_| "[]".to_string());
    let script = format!(
        r#"
const {{ KicadToCircuitJsonConverter }} = require("kicad-to-circuit-json");
const fs = require("fs");
const path = require("path");
const paths = {};
const converter = new KicadToCircuitJsonConverter();
for (const p of paths) {{
    try {{
        converter.addFile(path.basename(p), fs.readFileSync(p, "utf-8"));
    }} catch (e) {{}}
}}
converter.runUntilFinished();
const out = converter.getOutput();
fs.writeFileSync("{}", JSON.stringify(out, null, 2));
"#,
        paths_json,
        out_json_path.display()
    );

    let node_path = assemble_node_path();
    let success = run_script_with_runners(&script, &node_path, &out_json_path);

    if success {
        let content = std::fs::read_to_string(&out_json_path).map_err(|e| e.to_string())?;
        return Ok((out_json_path, content));
    }

    if let Some(fb) = fallback {
        println!(
            "⚠️  KiCad converter runner unavailable; falling back to existing \"{}\"...",
            fb.display()
        );
        let content = std::fs::read_to_string(&fb).map_err(|e| e.to_string())?;
        return Ok((fb, content));
    }

    Err("failed to convert KiCad project to Circuit JSON: runner or kicad-to-circuit-json not found".to_string())
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

fn assemble_node_path() -> String {
    let mut node_paths = Vec::new();
    if let Ok(np) = std::env::var("NODE_PATH") {
        node_paths.push(np);
    }
    node_paths.push("node_modules".to_string());
    node_paths.push("../node_modules".to_string());
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        node_paths.push(format!("{home}/.bun/install/global/node_modules"));
        node_paths.push(format!("{home}/.npm-global/lib/node_modules"));
        node_paths.push(format!("{home}/node_modules"));
    }
    node_paths.join(":")
}

fn run_script_with_runners(script: &str, node_path: &str, out_file: &Path) -> bool {
    let runners = ["bun", "node"];
    for runner_bin in runners {
        let mut cmd = std::process::Command::new(runner_bin);
        cmd.arg("-e").arg(script);
        cmd.env("NODE_PATH", node_path);

        if let Ok(out) = cmd.output() {
            if out.status.success() && out_file.is_file() {
                return true;
            }
        }
    }
    false
}

fn find_fallback_circuit_json(entry: &Path) -> Option<PathBuf> {
    let dist_json = PathBuf::from("dist/circuit.json");
    if dist_json.is_file() {
        return Some(dist_json);
    }
    let local_json = PathBuf::from("circuit.json");
    if local_json.is_file() {
        return Some(local_json);
    }
    if entry.is_dir() {
        let in_dir = entry.join("circuit.json");
        if in_dir.is_file() {
            return Some(in_dir);
        }
    }
    None
}
