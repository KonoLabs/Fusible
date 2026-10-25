//! Integration tests for `fusible assembly`: config errors, the fuse gate, and fail-closed modeling.
//!
//! A full build needs a tscircuit runner and a TSX project, so these tests stop before any
//! model is exported and run with an empty `PATH`.

mod common;

use std::path::Path;

use common::{fusible, require_bun, TestDir};

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

#[test]
fn a_generated_enclosure_assembles_to_a_glb_and_a_step_file() {
    if require_bun().is_none() {
        eprintln!("skipped: bun is not on PATH");
        return;
    }
    let dir = project(
        "assembly_success",
        "boards:\n  main: { entry: circuit.json }\n\
         bodies:\n  case: { generate: enclosure, board: main }\n\
         assembly: { parts: { case: { at: [0, 0, 0] } } }\n",
    );
    let circuit = dir.path.join("circuit.json");
    let source = std::fs::read_to_string(&circuit).unwrap();
    let outline =
        r#"[{"type":"pcb_board","width":40,"height":30,"center":{"x":0,"y":0},"thickness":1.4},"#;
    std::fs::write(&circuit, source.replacen('[', outline, 1)).unwrap();
    let out = fusible()
        .current_dir(&dir.path)
        .args(["assembly", "-f", "all"])
        .output()
        .expect("binary should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    let glb = std::fs::read(dir.path.join("dist/assembly.glb")).unwrap();
    assert_eq!(&glb[..4], b"glTF");
    let step = std::fs::read_to_string(dir.path.join("dist/assembly.step")).unwrap();
    assert!(step.contains("ISO-10303-21"));
    let report = std::fs::read_to_string(dir.path.join("dist/assembly/check.json")).unwrap();
    assert!(report.contains("\"fuse\": \"intact\""), "{report}");
}
