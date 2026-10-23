//! Integration tests for `fusible body eject` and `fusible body kit`.

mod common;

use std::path::Path;

use common::{fusible, TestDir};

const MANIFEST: &str = "\
boards:
  main: { entry: circuit.json }
bodies:
  lid: { generate: enclosure, board: main }
  case: { source: bodies/case.step }
";

const BOARD: &str = r#"[
  {"type":"pcb_board","width":40,"height":30,"center":{"x":0,"y":0},"thickness":1.4},
  {"type":"pcb_hole","x":-15,"y":10,"hole_diameter":3.2},
  {"type":"source_component","source_component_id":"sc1","name":"J1"},
  {"type":"pcb_component","pcb_component_id":"pc1","source_component_id":"sc1",
   "center":{"x":18,"y":0},"width":6,"height":8,"layer":"top"}
]"#;

fn project(name: &str) -> TestDir {
    let dir = TestDir::new(name);
    std::fs::write(dir.path.join("fusible.yml"), MANIFEST).unwrap();
    std::fs::write(dir.path.join("circuit.json"), BOARD).unwrap();
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
fn eject_refuses_to_overwrite_and_leaves_the_manifest_alone() {
    let dir = project("eject_exists");
    std::fs::create_dir_all(dir.path.join("bodies")).unwrap();
    let mine = dir.path.join("bodies/lid.step");
    std::fs::write(&mine, "my edits").unwrap();

    let (code, stderr) = run(&dir.path, &["body", "eject", "lid", "--skip-lint"]);
    assert_eq!(code, Some(2));
    assert!(stderr.contains("fusible::body::exists"), "{stderr}");
    assert_eq!(std::fs::read_to_string(&mine).unwrap(), "my edits");
    assert_eq!(
        std::fs::read_to_string(dir.path.join("fusible.yml")).unwrap(),
        MANIFEST
    );
}

#[test]
fn eject_rejects_authored_unknown_and_badly_named_targets() {
    let dir = project("eject_bad");
    for (args, needle) in [
        (vec!["body", "eject", "case"], "authored"),
        (vec!["body", "eject", "nope"], "no body named"),
        (
            vec!["body", "eject", "lid", "-o", "x.iges"],
            "unknown body format",
        ),
    ] {
        let (code, stderr) = run(&dir.path, &args);
        assert_eq!(code, Some(2), "{args:?}");
        assert!(stderr.contains(needle), "{args:?}: {stderr}");
    }
    assert!(!dir.path.join("bodies/lid.step").exists());
}

#[test]
fn kit_writes_kit_json_and_notes_a_missing_runner() {
    let dir = project("kit_ok");
    let (code, _) = run(&dir.path, &["body", "kit", "main", "--skip-lint"]);
    assert_eq!(code, Some(0));
    let text = std::fs::read_to_string(dir.path.join("bodies/kit/main/kit.json")).unwrap();
    let kit: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(kit["board"], "main");
    assert_eq!(kit["holes"][0]["diameter"], 3.2);
    assert_eq!(kit["openings"][0]["component"], "J1");
    assert_eq!(kit["openings"][0]["side"], "+x");
    assert!(!dir.path.join("bodies/kit/main/board.step").exists());
}

#[test]
fn kit_honors_the_output_directory_and_the_fuse_gate() {
    let dir = project("kit_gate");
    let custom = dir.path.join("out");
    let (code, _) = run(
        &dir.path,
        &[
            "body",
            "kit",
            "main",
            "--skip-lint",
            "-o",
            custom.to_str().unwrap(),
        ],
    );
    assert_eq!(code, Some(0));
    assert!(custom.join("kit.json").is_file());

    let blown = TestDir::new("kit_blown");
    let target = blown.path.join("kit");
    let floating = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/circuit-floating.json");
    let (code, _) = run(
        &blown.path,
        &[
            "body",
            "kit",
            floating.to_str().unwrap(),
            "-o",
            target.to_str().unwrap(),
        ],
    );
    assert_eq!(code, Some(1));
    assert!(!target.exists());
}

#[test]
fn body_keeps_its_aliases_and_plain_target_form() {
    let dir = project("body_alias");
    std::fs::create_dir_all(dir.path.join("bodies")).unwrap();
    std::fs::write(dir.path.join("bodies/case.step"), "solid").unwrap();
    for args in [["case", "kit", "--help"], ["mechanical", "eject", "--help"]] {
        assert_eq!(run(&dir.path, &args).0, Some(0), "{args:?}");
    }
    let (code, stderr) = run(&dir.path, &["body", "case", "--skip-lint"]);
    assert_eq!(code, Some(0), "{stderr}");
}

#[test]
fn kit_of_a_bare_circuit_file_is_named_after_the_file() {
    let dir = TestDir::new("kit_bare");
    std::fs::write(dir.path.join("sensor.json"), BOARD).unwrap();
    let (code, stderr) = run(&dir.path, &["body", "kit", "sensor.json", "--skip-lint"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(dir.path.join("bodies/kit/sensor/kit.json").is_file());
    assert!(!dir.path.join("bodies/kit/board").exists());
}
