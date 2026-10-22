use super::diff::{compare_kicad_with_circuit, DiffKind};
use super::sexpr::{extract_footprints, parse_sexpr};
use super::sync::apply_kicad_sync;

fn placed(name: &str, x: f64, y: f64, rotation: f64) -> String {
    format!(
        r#"{{ "type": "source_component", "source_component_id": "sc_{name}", "name": "{name}" }},
        {{ "type": "pcb_component", "pcb_component_id": "pc_{name}", "source_component_id": "sc_{name}",
          "center": {{ "x": {x}, "y": {y} }}, "rotation": {rotation}, "layer": "top",
          "width": 1, "height": 1 }}"#
    )
}

fn elements(json: &str) -> Vec<serde_json::Value> {
    super::diff::parse_circuit_elements(json).expect("valid circuit json")
}

fn temp_path(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "fusible_test_{tag}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

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
    let kicad_fps = extract_footprints(&parse_sexpr(sexpr_str).unwrap());

    let circuit_json = format!("[{}]", placed("R1", 10.0, -20.0, 0.0));

    let report = compare_kicad_with_circuit(&kicad_fps, &elements(&circuit_json));
    assert_eq!(report.diffs.len(), 1);
    let diff = &report.diffs[0];
    assert_eq!(diff.designator, "R1");
    assert_eq!(diff.current_pos, Some([10.0, -20.0]));
    match diff.kind {
        DiffKind::Rotated { d_rot } => assert!((d_rot - 45.0).abs() < 1e-4),
        _ => panic!("expected rotation diff, got {:?}", diff.kind),
    }
}

#[test]
fn rotation_compare_is_modulo_360_and_report_is_sorted() {
    let sexpr_str = r#"
        (kicad_pcb
          (footprint "R" (layer "F.Cu") (at 0 0 720) (property "Reference" "R2"))
          (footprint "R" (layer "F.Cu") (at 0 0 -90) (property "Reference" "R1"))
        )
    "#;
    let kicad_fps = extract_footprints(&parse_sexpr(sexpr_str).unwrap());
    let circuit_json = format!(
        "[{}, {}, {}]",
        placed("R1", 0.0, 0.0, 270.0),
        placed("R2", 0.0, 0.0, 0.0),
        placed("R0", 5.0, 5.0, 0.0)
    );

    let report = compare_kicad_with_circuit(&kicad_fps, &elements(&circuit_json));
    let names: Vec<&str> = report.diffs.iter().map(|d| d.designator.as_str()).collect();
    assert_eq!(names, ["R0", "R1", "R2"]);
    assert_eq!(report.diffs[1].kind, DiffKind::Unchanged);
    assert_eq!(report.diffs[2].kind, DiffKind::Unchanged);
    assert_eq!(report.diffs[0].kind, DiffKind::MissingInKicad);
}

#[test]
fn applies_sync_to_manual_edits() {
    let temp_dir = temp_path("sync");
    let manual_edits_path = temp_dir.join("manual-edits.json");

    let sexpr_str = r#"
        (kicad_pcb
          (footprint "R" (layer "F.Cu") (at 15.0 25.0) (property "Reference" "R1"))
        )
    "#;
    let kicad_fps = extract_footprints(&parse_sexpr(sexpr_str).unwrap());
    let circuit_json = format!("[{}]", placed("R1", 0.0, 0.0, 0.0));

    let report = compare_kicad_with_circuit(&kicad_fps, &elements(&circuit_json));
    let msg = apply_kicad_sync(&report, &manual_edits_path, false).expect("sync should succeed");

    assert!(msg.contains("Synced 1 component placement"));
    let content = std::fs::read_to_string(&manual_edits_path).unwrap();
    assert!(content.contains("\"selector\": \"R1\""));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn sync_replaces_a_non_object_center_instead_of_panicking() {
    let temp_dir = temp_path("sync_center");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let manual_edits_path = temp_dir.join("manual-edits.json");
    std::fs::write(
        &manual_edits_path,
        r#"{ "pcb_placements": [{ "selector": "R1", "center": 5 }], "edit_events": [] }"#,
    )
    .unwrap();

    let sexpr_str =
        r#"(kicad_pcb (footprint "R" (layer "F.Cu") (at 15.0 25.0) (property "Reference" "R1")))"#;
    let kicad_fps = extract_footprints(&parse_sexpr(sexpr_str).unwrap());
    let report = compare_kicad_with_circuit(
        &kicad_fps,
        &elements(&format!("[{}]", placed("R1", 0.0, 0.0, 0.0))),
    );
    apply_kicad_sync(&report, &manual_edits_path, false).expect("sync should succeed");

    let root: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manual_edits_path).unwrap()).unwrap();
    assert!(root["pcb_placements"][0]["center"]["x"].is_f64());

    let _ = std::fs::remove_dir_all(&temp_dir);
}
