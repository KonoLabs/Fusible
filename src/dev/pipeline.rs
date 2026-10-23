use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde::Serialize;

use crate::commands::check::load_circuit_source;
use crate::commands::target::board_for_entry;
use crate::linter::{LintContext, Linter, Severity};
use crate::netlist::Netlist;
use crate::parser::{self, Circuit};

#[derive(Debug, Clone, Serialize)]
pub struct DevDiagnostic {
    pub rule: String,
    pub message: String,
    pub help: Option<String>,
}

/// The studio payload pushed over SSE and served at `/api/circuit`.
///
/// `board` is the manifest name of the board, absent for a file outside the manifest.
///
/// `revision` is a 16-hex-char hash of the Circuit JSON source text, or empty when
/// the source could not be loaded.
#[derive(Debug, Clone, Serialize)]
pub struct DevPayload {
    pub board_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board: Option<String>,
    pub fuse_blown: bool,
    pub diagnostics: Vec<DevDiagnostic>,
    pub circuit_json: Option<serde_json::Value>,
    pub revision: String,
}

/// Builds the studio payload for `entry`, returning the Circuit JSON source text alongside it.
pub fn execute_pipeline(entry: &Path) -> (DevPayload, Option<String>) {
    let board = board_for_entry(entry);
    let (mut payload, source) = match load_circuit_source(entry) {
        Ok((json_path, source)) => {
            let payload = analyze(board.title, board.mcu.as_deref(), &json_path, &source);
            (payload, Some(source))
        }
        Err(err) => (source_failure(board.title, err.to_string()), None),
    };
    payload.board = board.name;
    (payload, source)
}

/// Hashes the Circuit JSON source text into a stable 16-hex-char revision.
pub fn revision(source: &str) -> String {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn diagnostic(rule: &str, message: String, help: &str) -> DevDiagnostic {
    DevDiagnostic {
        rule: rule.to_string(),
        message,
        help: Some(help.to_string()),
    }
}

fn source_failure(board_name: String, message: String) -> DevPayload {
    DevPayload {
        board_name,
        board: None,
        fuse_blown: true,
        diagnostics: vec![diagnostic(
            "source_error",
            message,
            "Ensure your circuit file compiles and is valid JSON/TSX",
        )],
        circuit_json: None,
        revision: String::new(),
    }
}

fn analyze(board_name: String, mcu: Option<&str>, json_path: &Path, source: &str) -> DevPayload {
    let mut payload = DevPayload {
        board_name,
        board: None,
        fuse_blown: true,
        diagnostics: Vec::new(),
        circuit_json: serde_json::from_str(source).ok(),
        revision: revision(source),
    };
    match parser::parse(source, json_path) {
        Ok(circuit) => lint(&mut payload, &circuit, mcu),
        Err(err) => payload.diagnostics.push(diagnostic(
            "parse_error",
            err.to_string(),
            "Check circuit JSON syntax",
        )),
    }
    payload
}

fn lint(payload: &mut DevPayload, circuit: &Circuit, mcu: Option<&str>) {
    let netlist = Netlist::from_circuit(circuit);
    let ctx = LintContext::new(circuit, &netlist).with_mcu(mcu);
    let violations = Linter::default().run_with_context(&ctx);
    payload.fuse_blown = violations.iter().any(|v| v.severity == Severity::Error);
    payload.diagnostics = violations
        .into_iter()
        .map(|v| DevDiagnostic {
            rule: v.rule,
            message: v.message,
            help: v.help,
        })
        .collect();
}
