//! Integration tests for fuse-gated `export` and `mechanical`.

mod common;

use common::{fusible, TestDir};

#[test]
fn subcommand_export_fuse_blown_blocks_export() {
    let out = fusible()
        .args(["export", "examples/circuit-floating.json"])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("the fuse blew"));
}

#[test]
fn subcommand_export_docs_generates_native_svgs() {
    let fixture = TestDir::new("export_docs_native");
    let out_dir = fixture.path.join("docs");

    let out = fusible()
        .args([
            "export",
            "examples/circuit.json",
            "--preset",
            "docs",
            "-o",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(0));
    assert!(out_dir.join("pcb.svg").exists());
    assert!(out_dir.join("schematic.svg").exists());

    let pcb_svg = std::fs::read_to_string(out_dir.join("pcb.svg")).unwrap();
    assert!(pcb_svg.contains("<svg"));
    assert!(pcb_svg.contains("substrate"));

    let sch_svg = std::fs::read_to_string(out_dir.join("schematic.svg")).unwrap();
    assert!(sch_svg.contains("<svg"));
    assert!(sch_svg.contains("components"));
}

#[test]
fn subcommand_export_rejects_unknown_preset() {
    let out = fusible()
        .args([
            "export",
            "examples/circuit.json",
            "--preset",
            "nonexistent",
            "--skip-lint",
        ])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unknown export preset"));
}

#[test]
fn subcommand_mechanical_fuse_blown_writes_nothing() {
    let dir = TestDir::new("mechanical_blown");
    let out_dir = dir.path.join("mech");
    let out = fusible()
        .args(["mechanical", "examples/circuit-floating.json", "-o"])
        .arg(&out_dir)
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(1));
    assert!(!out_dir.exists());
}

#[test]
fn subcommand_mechanical_rejects_bad_input_as_tool_error() {
    for args in [
        vec!["mechanical", "examples/circuit.json", "-f", "iges"],
        vec!["mechanical", "examples/circuit.json", "--skip-lint", "-o"],
    ] {
        let dir = TestDir::new("mechanical_bad");
        let out = fusible()
            .args(&args)
            .args(args.ends_with(&["-o"]).then(|| dir.path.join("mech")))
            .output()
            .expect("binary should run");

        assert_eq!(out.status.code(), Some(2), "args: {args:?}");
    }
}
