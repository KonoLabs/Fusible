//! Integration tests for subcommands (`check`, `init`, etc.).

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

fn fusible() -> Command {
    Command::new(env!("CARGO_BIN_EXE_fusible"))
}

struct TestDir {
    pub path: std::path::PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let path = std::env::temp_dir().join(format!("fusible_integration_{pid}_{id}_{name}"));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("failed to create test dir");
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[test]
fn subcommand_check_validates_intact_circuit() {
    let out = fusible()
        .args(["check", "examples/circuit.json", "--mcu", "stm32g431"])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("fuse intact"));
    assert!(stdout.contains("stm32g431"));
}

#[test]
fn subcommand_check_detects_blown_fuse() {
    let out = fusible()
        .args(["check", "examples/circuit-floating.json"])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("the fuse blew"));
}

#[test]
fn subcommand_check_blows_fuse_on_silicon_pin_mux_conflict() {
    let out = fusible()
        .args([
            "check",
            "examples/circuit-pinmux-violation.json",
            "--mcu",
            "stm32g431",
        ])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("the fuse blew"));
    assert!(stderr.contains("PA5"));
    assert!(stderr.contains("UART1_TX"));
}

#[test]
fn subcommand_init_creates_valid_manifest() {
    let fixture = TestDir::new("subcmd_init");

    let out = fusible()
        .args(["init", "--name", "test-project", "--no-install"])
        .current_dir(&fixture.path)
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(0));
    assert!(fixture.path.join("fusible.yml").exists());
    assert!(fixture.path.join("hardware/index.tsx").exists());
    assert!(fixture.path.join("hardware/package.json").exists());
    assert!(fixture.path.join("hardware/tsconfig.json").exists());

    let content = std::fs::read_to_string(fixture.path.join("fusible.yml")).unwrap();
    assert!(content.contains("name: test-project"));
}

#[test]
fn subcommand_init_with_path_and_mcu() {
    let fixture = TestDir::new("subcmd_init_path");
    let target = fixture.path.join("sensor_node");

    let out = fusible()
        .args([
            "init",
            target.to_str().unwrap(),
            "--name",
            "sensor-node",
            "--mcu",
            "stm32g431",
            "--no-install",
        ])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(0));
    assert!(target.join("fusible.yml").exists());
    assert!(target.join("hardware/index.tsx").exists());
    assert!(target.join("hardware/package.json").exists());

    let manifest = std::fs::read_to_string(target.join("fusible.yml")).unwrap();
    assert!(manifest.contains("name: sensor-node"));
    assert!(manifest.contains("chip: stm32g431"));
}

#[test]
fn runs_check_from_manifest_with_no_arguments() {
    let fixture = TestDir::new("manifest_check");
    let circuit_json = include_str!("../examples/circuit.json");
    std::fs::write(fixture.path.join("board.json"), circuit_json).unwrap();

    let manifest = r#"
name: auto-check
hardware:
  entry: board.json
"#;
    std::fs::write(fixture.path.join("fusible.yml"), manifest).unwrap();

    let out = fusible()
        .current_dir(&fixture.path)
        .output()
        .expect("binary should run");

    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stdout: {stdout}, stderr: {stderr}"
    );
    assert!(stdout.contains("fuse intact"));
}

#[test]
fn subcommand_doctor_reports_health() {
    let out = fusible().arg("doctor").output().expect("binary should run");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Fusible Toolchain Doctor"));
    assert!(stdout.contains("Runtime prerequisites"));
}

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
fn subcommand_route_dry_run_on_routable_circuit() {
    let fixture = TestDir::new("subcmd_route");
    let circuit = r#"[
        { "type": "pcb_smtpad", "x": 0.0, "y": 0.0, "width": 1.0, "height": 1.0, "layer": "top", "net_id": "GND" },
        { "type": "pcb_smtpad", "x": 10.0, "y": 0.0, "width": 1.0, "height": 1.0, "layer": "top", "net_id": "GND" }
    ]"#;
    let json_file = fixture.path.join("circuit.json");
    std::fs::write(&json_file, circuit).unwrap();

    let out = fusible()
        .args(["route", json_file.to_str().unwrap(), "--dry-run"])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Autorouting complete: 1/1 net(s) routed"));
    assert!(stdout.contains("Dry run complete"));
}

#[test]
fn subcommand_diff_and_sync_kicad() {
    let fixture = TestDir::new("subcmd_diff_sync");
    let kicad_pcb = fixture.path.join("board.kicad_pcb");
    let sexpr = r#"(kicad_pcb
        (footprint "R" (layer "F.Cu") (at 10.0 20.0 90) (property "Reference" "R1"))
    )"#;
    std::fs::write(&kicad_pcb, sexpr).unwrap();

    let circuit_file = fixture.path.join("circuit.json");
    let circuit = r#"[
        { "type": "source_component", "source_component_id": "sc_r1", "name": "R1" },
        { "type": "pcb_component", "pcb_component_id": "pc_r1", "source_component_id": "sc_r1", "center": { "x": 0.0, "y": 0.0 }, "rotation": 0.0, "layer": "top" }
    ]"#;
    std::fs::write(&circuit_file, circuit).unwrap();

    let diff_out = fusible()
        .args([
            "diff",
            kicad_pcb.to_str().unwrap(),
            circuit_file.to_str().unwrap(),
        ])
        .output()
        .expect("binary should run");

    assert_eq!(diff_out.status.code(), Some(1));
    let diff_stdout = String::from_utf8_lossy(&diff_out.stdout);
    assert!(diff_stdout.contains("R1"));
    assert!(diff_stdout.contains("Rotated"));

    let manual_edits = fixture.path.join("manual-edits.json");
    let sync_out = fusible()
        .args([
            "sync",
            kicad_pcb.to_str().unwrap(),
            circuit_file.to_str().unwrap(),
            "--target",
            manual_edits.to_str().unwrap(),
            "--write",
        ])
        .output()
        .expect("binary should run");

    assert_eq!(sync_out.status.code(), Some(0));
    assert!(manual_edits.is_file());
    let edits_content = std::fs::read_to_string(&manual_edits).unwrap();
    assert!(edits_content.contains("\"selector\": \"R1\""));
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

#[test]
fn bare_invocation_without_manifest_is_a_tool_error() {
    let dir = TestDir::new("bare");
    let out = fusible()
        .current_dir(&dir.path)
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(2));
}
