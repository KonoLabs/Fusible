use super::diff::{compare_kicad_with_circuit, DiffKind};
use super::sexpr::{extract_footprints, parse_sexpr};
use super::sync::apply_kicad_sync;

#[test]
fn parses_kicad6_footprint_sexpr() {
    let sexpr_str = r#"
        (kicad_pcb (version 20221018) (generator pcbnew)
          (footprint "Resistor_SMD:R_0603_1608Metric"
            (layer "F.Cu")
            (at 125.5 84.2 90)
            (property "Reference" "R1")
            (property "Value" "10k")
          )
          (footprint "Capacitor_SMD:C_0402_1005Metric"
            (layer "B.Cu")
            (at 110.0 95.0)
            (property "Reference" "C1")
          )
        )
    "#;

    let sexpr = parse_sexpr(sexpr_str).expect("S-expression should parse cleanly");
    let footprints = extract_footprints(&sexpr);

    assert_eq!(footprints.len(), 2);
    assert_eq!(footprints[0].designator, "R1");
    assert_eq!(footprints[0].x, 125.5);
    assert_eq!(footprints[0].y, 84.2);
    assert_eq!(footprints[0].rotation, 90.0);
    assert_eq!(footprints[0].layer, "top");

    assert_eq!(footprints[1].designator, "C1");
    assert_eq!(footprints[1].x, 110.0);
    assert_eq!(footprints[1].y, 95.0);
    assert_eq!(footprints[1].rotation, 0.0);
    assert_eq!(footprints[1].layer, "bottom");
}

#[test]
fn compares_and_detects_layout_drift() {
    let sexpr_str = r#"
        (kicad_pcb
          (footprint "R" (layer "F.Cu") (at 10.0 20.0 45) (property "Reference" "R1"))
        )
    "#;
    let sexpr = parse_sexpr(sexpr_str).unwrap();
    let kicad_fps = extract_footprints(&sexpr);

    let circuit_json = r#"[
        { "type": "source_component", "name": "R1", "x": 10.0, "y": -20.0, "rotation": 0.0, "layer": "top" }
    ]"#;

    let report = compare_kicad_with_circuit(&kicad_fps, circuit_json);
    assert_eq!(report.diffs.len(), 1);
    let diff = &report.diffs[0];
    assert_eq!(diff.designator, "R1");
    match diff.kind {
        DiffKind::Rotated { d_rot } => {
            assert!((d_rot - 45.0).abs() < 1e-4);
        }
        _ => panic!("expected rotation diff"),
    }
}

#[test]
fn applies_sync_to_manual_edits() {
    let temp_dir = std::env::temp_dir().join(format!(
        "fusible_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let manual_edits_path = temp_dir.join("manual-edits.json");

    let sexpr_str = r#"
        (kicad_pcb
          (footprint "R" (layer "F.Cu") (at 15.0 25.0) (property "Reference" "R1"))
        )
    "#;
    let sexpr = parse_sexpr(sexpr_str).unwrap();
    let kicad_fps = extract_footprints(&sexpr);

    let circuit_json = r#"[
        { "type": "source_component", "name": "R1", "x": 0.0, "y": 0.0, "rotation": 0.0, "layer": "top" }
    ]"#;

    let report = compare_kicad_with_circuit(&kicad_fps, circuit_json);
    let msg = apply_kicad_sync(&report, &manual_edits_path, false).expect("sync should succeed");

    assert!(msg.contains("Synced 1 component placement"));
    assert!(manual_edits_path.is_file());

    let content = std::fs::read_to_string(&manual_edits_path).unwrap();
    assert!(content.contains("\"selector\": \"R1\""));

    let _ = std::fs::remove_dir_all(&temp_dir);
}
