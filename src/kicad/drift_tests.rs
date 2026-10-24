use super::diff::{compare_kicad_with_circuit, parse_circuit_elements, DiffKind};
use super::sexpr::{extract_footprints, parse_sexpr};
use super::sync::apply_kicad_sync;

fn part(name: &str, x: f64, y: f64) -> String {
    format!(
        r#"{{ "type": "source_component", "source_component_id": "sc_{name}", "name": "{name}" }},
        {{ "type": "pcb_component", "pcb_component_id": "pc_{name}", "source_component_id": "sc_{name}",
          "center": {{ "x": {x}, "y": {y} }}, "rotation": 0, "layer": "top", "width": 1, "height": 1 }}"#
    )
}

fn footprint(name: &str, x: f64, y: f64) -> String {
    format!(r#"(footprint "R" (layer "F.Cu") (at {x} {y}) (property "Reference" "{name}"))"#)
}

fn circuit(names: &[String]) -> String {
    let parts: Vec<String> = names
        .iter()
        .enumerate()
        .map(|(i, n)| part(n, i as f64 * 2.0, 0.0))
        .collect();
    format!("[{}]", parts.join(","))
}

fn pcb(names: &[String], moved: &str, extra: &str) -> String {
    let fps: Vec<String> = names
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let shift = if n == moved { 10.0 } else { 0.0 };
            footprint(n, 100.0 + i as f64 * 2.0 + shift, 50.0)
        })
        .collect();
    format!("(kicad_pcb {} {extra})", fps.join(" "))
}

fn names(count: usize) -> Vec<String> {
    (1..=count).map(|i| format!("R{i}")).collect()
}

#[test]
fn one_moved_part_among_many_does_not_drag_the_others() {
    let names = names(20);
    let fps = extract_footprints(&parse_sexpr(&pcb(&names, "R7", "")).unwrap());
    let report =
        compare_kicad_with_circuit(&fps, &parse_circuit_elements(&circuit(&names)).unwrap());
    assert_eq!(report.modified_count, 1);
    let moved = report.diffs.iter().find(|d| d.designator == "R7").unwrap();
    assert!(matches!(moved.kind, DiffKind::Moved { .. }));
}

#[test]
fn sync_writes_only_parts_that_moved() {
    let names = names(3);
    let extra = footprint("R9", 5.0, 5.0);
    let fps = extract_footprints(&parse_sexpr(&pcb(&names, "R2", &extra)).unwrap());
    let report =
        compare_kicad_with_circuit(&fps, &parse_circuit_elements(&circuit(&names)).unwrap());
    let dir = std::env::temp_dir().join(format!("fusible_drift_{}", std::process::id()));
    let path = dir.join("manual-edits.json");
    let msg = apply_kicad_sync(&report, &path, false).unwrap();
    let written = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(msg.contains("Synced 1 "), "{msg}");
    assert!(written.contains("\"R2\"") && !written.contains("R9") && !written.contains("\"R1\""));
}
