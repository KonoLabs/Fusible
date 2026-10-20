use std::path::Path;

use serde::Serialize;

use crate::commands::check::{load_circuit_source, resolve_mcu};
use crate::config::FusibleConfig;
use crate::linter::{LintContext, Linter};
use crate::netlist::Netlist;
use crate::parser;
use crate::svg::{render_pcb_svg, render_schematic_svg};

#[derive(Debug, Clone, Serialize)]
pub struct DevDiagnostic {
    pub rule: String,
    pub message: String,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DevPayload {
    pub board_name: String,
    pub fuse_blown: bool,
    pub diagnostics: Vec<DevDiagnostic>,
    pub pcb_svg: String,
    pub schematic_svg: String,
    pub circuit_json: Option<serde_json::Value>,
}

pub fn execute_pipeline(entry: &Path) -> DevPayload {
    let board_name = FusibleConfig::find_and_load(Path::new("."))
        .ok()
        .flatten()
        .and_then(|(_, cfg)| cfg.name)
        .unwrap_or_else(|| {
            entry
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Board".to_string())
        });

    let (json_path, source) = match load_circuit_source(entry) {
        Ok(res) => res,
        Err(err) => {
            return DevPayload {
                board_name,
                fuse_blown: true,
                diagnostics: vec![DevDiagnostic {
                    rule: "source_error".to_string(),
                    message: err.to_string(),
                    help: Some(
                        "Ensure your circuit file compiles and is valid JSON/TSX".to_string(),
                    ),
                }],
                pcb_svg: String::new(),
                schematic_svg: String::new(),
                circuit_json: None,
            };
        }
    };

    let circuit_json = serde_json::from_str::<serde_json::Value>(&source).ok();
    let pcb_svg = render_pcb_svg(&source);

    let circuit = match parser::parse(&source, &json_path) {
        Ok(c) => c,
        Err(err) => {
            return DevPayload {
                board_name,
                fuse_blown: true,
                diagnostics: vec![DevDiagnostic {
                    rule: "parse_error".to_string(),
                    message: err.to_string(),
                    help: Some("Check circuit JSON syntax".to_string()),
                }],
                pcb_svg,
                schematic_svg: String::new(),
                circuit_json,
            };
        }
    };

    let netlist = Netlist::from_circuit(&circuit);
    let schematic_svg = render_schematic_svg(&circuit, &netlist);

    let target_mcu = resolve_mcu(None);
    let ctx = LintContext::new(&circuit, &netlist).with_mcu(target_mcu.as_deref());
    let violations = Linter::default().run_with_context(&ctx);

    let fuse_blown = !violations.is_empty();
    let diagnostics = violations
        .into_iter()
        .map(|v| DevDiagnostic {
            rule: v.rule,
            message: v.message,
            help: v.help,
        })
        .collect();

    DevPayload {
        board_name,
        fuse_blown,
        diagnostics,
        pcb_svg,
        schematic_svg,
        circuit_json,
    }
}
