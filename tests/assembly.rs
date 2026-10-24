//! Integration tests for `fusible assembly`: config errors, the fuse gate, and fail-closed modeling.
//!
//! A full build needs a tscircuit runner and a TSX project, so these tests stop before any
//! model is exported and run with an empty `PATH`.

mod common;

use std::path::Path;

use common::{fusible, TestDir};

fn project(name: &str, manifest: &str) -> TestDir {
    let dir = TestDir::new(name);
    std::fs::write(dir.path.join("fusible.yml"), manifest).unwrap();
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    for file in ["circuit.json", "circuit-floating.json"] {
        std::fs::copy(examples.join(file), dir.path.join(file)).unwrap();
    }
    dir
}

fn run(dir: &Path, args: &[&str]) -> (Option<i32>, String) {
    let out = fusible()
        .current_dir(dir)
        .env("PATH", "")
        .args(args)
        .output()
        .expect("binary should run");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn a_manifest_without_assembly_or_with_a_bad_format_is_a_tool_error() {
    let dir = project(
        "assembly_missing",
        "boards:\n  main: { entry: circuit.json }\n",
    );
    let (code, stderr) = run(&dir.path, &["assembly"]);
    assert_eq!(code, Some(2));
    assert!(stderr.contains("fusible::assembly::missing"), "{stderr}");

    let dir = project(
        "assembly_format",
        "boards:\n  main: { entry: circuit.json }\nassembly: { parts: { main: {} } }\n",
    );
    let (code, stderr) = run(&dir.path, &["assembly", "-f", "iges"]);
    assert_eq!(code, Some(2));
    assert!(stderr.contains("unknown assembly format"), "{stderr}");
}

#[test]
fn a_board_without_a_tscircuit_source_cannot_be_modeled() {
    let dir = project(
        "assembly_json_board",
        "boards:\n  main: { entry: circuit.json }\nassembly: { parts: { main: {} } }\n",
    );
    let (code, stderr) = run(&dir.path, &["assembly"]);
    assert_eq!(code, Some(2));
    assert!(
        stderr.contains("fusible::assembly::no_board_model"),
        "{stderr}"
    );
    assert!(!dir.path.join("dist").exists());
}

#[test]
fn a_blown_fuse_on_a_body_board_stops_the_assembly_before_any_file() {
    let dir = project(
        "assembly_blown",
        "boards:\n  main: { entry: circuit-floating.json }\n\
         bodies:\n  case: { generate: enclosure, board: main }\n\
         assembly: { parts: { case: { at: [0, 0, 0] } } }\n",
    );
    let (code, _) = run(&dir.path, &["assembly"]);
    assert_eq!(code, Some(1));
    assert!(!dir.path.join("dist").exists());
}

#[test]
fn an_unplaceable_part_is_rejected_when_the_manifest_loads() {
    let dir = project(
        "assembly_unplaceable",
        "boards:\n  main: { entry: circuit.json }\n\
         bodies:\n  bracket: { source: bracket.step }\n\
         assembly: { parts: { bracket: {} } }\n",
    );
    let (code, stderr) = run(&dir.path, &["assembly"]);
    assert_eq!(code, Some(2));
    assert!(stderr.contains("body without a board"), "{stderr}");
}
