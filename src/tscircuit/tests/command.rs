//! Unit tests for runner command generation and diagnostics.

use std::path::{Path, PathBuf};

use crate::tscircuit::command::{build_command, dev_command, export_command};
use crate::tscircuit::error::RunnerError;
use crate::tscircuit::output::extract_circuit_json_path;
use crate::tscircuit::RunnerKind;

#[test]
fn dev_command_construction_without_port() {
    let kind = RunnerKind::Local(PathBuf::from("./node_modules/.bin/tsci"));
    let cmd = dev_command(&kind, Path::new("circuit.tsx"), None);

    assert_eq!(cmd.get_program(), "./node_modules/.bin/tsci");
    let args: Vec<_> = cmd.get_args().collect();
    assert_eq!(args, &["dev", "circuit.tsx"]);
}

#[test]
fn dev_command_construction_with_port_and_bun() {
    let kind = RunnerKind::Bun(PathBuf::from("/usr/bin/bun"));
    let cmd = dev_command(&kind, Path::new("src/main.tsx"), Some(3020));

    assert_eq!(cmd.get_program(), "/usr/bin/bun");
    let args: Vec<_> = cmd.get_args().collect();
    assert_eq!(args, &["x", "tscircuit", "dev", "src/main.tsx", "-p", "3020"]);
}

#[test]
fn export_command_construction_without_output() {
    let kind = RunnerKind::Global(PathBuf::from("/usr/local/bin/tsci"));
    let cmd = export_command(&kind, Path::new("circuit.tsx"), "circuit-json", None);

    assert_eq!(cmd.get_program(), "/usr/local/bin/tsci");
    let args: Vec<_> = cmd.get_args().collect();
    assert_eq!(args, &["export", "circuit.tsx", "-f", "circuit-json"]);
}

#[test]
fn export_command_construction_with_output_and_npx() {
    let kind = RunnerKind::Npx(PathBuf::from("/usr/bin/npx"));
    let out = Path::new("dist/export.json");
    let cmd = export_command(&kind, Path::new("circuit.tsx"), "gerbers", Some(out));

    assert_eq!(cmd.get_program(), "/usr/bin/npx");
    let args: Vec<_> = cmd.get_args().collect();
    assert_eq!(
        args,
        &["tscircuit", "export", "circuit.tsx", "-f", "gerbers", "-o", "dist/export.json"]
    );
}

#[test]
fn build_command_construction() {
    let kind = RunnerKind::Bun(PathBuf::from("/usr/bin/bun"));
    let cmd = build_command(&kind, Path::new("circuit.tsx"));

    assert_eq!(cmd.get_program(), "/usr/bin/bun");
    let args: Vec<_> = cmd.get_args().collect();
    assert_eq!(args, &["x", "tscircuit", "build", "circuit.tsx"]);
}

#[test]
fn extracts_circuit_json_path_from_stdout() {
    let stdout = "Circuit JSON written to dist/circuit/circuit.json\n";
    let extracted = extract_circuit_json_path(stdout);
    assert_eq!(extracted, Some(PathBuf::from("dist/circuit/circuit.json")));
}

#[test]
fn runner_error_diagnostics() {
    use miette::Diagnostic;

    let not_found = RunnerError::NotFound;
    assert_eq!(
        not_found.code().map(|c| c.to_string()),
        Some("fusible::tscircuit::not_found".to_string())
    );
    assert!(not_found.help().is_some());
}
