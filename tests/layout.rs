//! Integration tests for `route`, `diff` and `sync`.

use std::path::{Path, PathBuf};

mod common;

use common::{fusible, TestDir};

const ROUTABLE_CIRCUIT: &str = r#"[
    { "type": "pcb_board", "width": 30.0, "height": 10.0, "center": { "x": 5.0, "y": 0.0 } },
    { "type": "source_port", "source_port_id": "sp1", "name": "1" },
    { "type": "source_port", "source_port_id": "sp2", "name": "2" },
    { "type": "source_trace", "source_trace_id": "st1",
      "connected_source_port_ids": ["sp1", "sp2"], "connected_source_net_ids": [] },
    { "type": "pcb_port", "pcb_port_id": "pp1", "source_port_id": "sp1", "x": 0.0, "y": 0.0 },
    { "type": "pcb_port", "pcb_port_id": "pp2", "source_port_id": "sp2", "x": 10.0, "y": 0.0 },
    { "type": "pcb_smtpad", "pcb_port_id": "pp1", "shape": "rect",
      "x": 0.0, "y": 0.0, "width": 1.0, "height": 1.0, "layer": "top" },
    { "type": "pcb_smtpad", "pcb_port_id": "pp2", "shape": "rect",
      "x": 10.0, "y": 0.0, "width": 1.0, "height": 1.0, "layer": "top" }
]"#;

const KICAD_PCB: &str = r#"(kicad_pcb
    (footprint "R" (layer "F.Cu") (at 10.0 20.0 90) (property "Reference" "R1"))
)"#;

const UNROTATED_CIRCUIT: &str = r#"[
    { "type": "source_component", "source_component_id": "sc_r1", "name": "R1" },
    { "type": "pcb_component", "pcb_component_id": "pc_r1", "source_component_id": "sc_r1",
      "center": { "x": 0.0, "y": 0.0 }, "rotation": 0.0, "layer": "top" }
]"#;

/// Writes the KiCad board and the drifted Circuit JSON, returning both paths.
fn write_drift_fixture(dir: &Path) -> (PathBuf, PathBuf) {
    let kicad_pcb = dir.join("board.kicad_pcb");
    std::fs::write(&kicad_pcb, KICAD_PCB).unwrap();
    let circuit_file = dir.join("circuit.json");
    std::fs::write(&circuit_file, UNROTATED_CIRCUIT).unwrap();
    (kicad_pcb, circuit_file)
}

#[test]
fn subcommand_route_dry_run_on_routable_circuit() {
    let fixture = TestDir::new("subcmd_route");
    let json_file = fixture.path.join("circuit.json");
    std::fs::write(&json_file, ROUTABLE_CIRCUIT).unwrap();

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
fn subcommand_diff_reports_drift() {
    let fixture = TestDir::new("subcmd_diff");
    let (kicad_pcb, circuit_file) = write_drift_fixture(&fixture.path);

    let diff_out = fusible()
        .arg("diff")
        .arg(&kicad_pcb)
        .arg(&circuit_file)
        .output()
        .expect("binary should run");

    assert_eq!(diff_out.status.code(), Some(1));
    let diff_stdout = String::from_utf8_lossy(&diff_out.stdout);
    assert!(diff_stdout.contains("R1"));
    assert!(diff_stdout.contains("Rotated"));
}

#[test]
fn subcommand_sync_writes_manual_edits() {
    let fixture = TestDir::new("subcmd_sync");
    let (kicad_pcb, circuit_file) = write_drift_fixture(&fixture.path);
    let manual_edits = fixture.path.join("manual-edits.json");

    let sync_out = fusible()
        .arg("sync")
        .arg(&kicad_pcb)
        .arg(&circuit_file)
        .arg("--target")
        .arg(&manual_edits)
        .arg("--write")
        .output()
        .expect("binary should run");

    assert_eq!(sync_out.status.code(), Some(0));
    assert!(manual_edits.is_file());
    let edits_content = std::fs::read_to_string(&manual_edits).unwrap();
    assert!(edits_content.contains("\"selector\": \"R1\""));
}
