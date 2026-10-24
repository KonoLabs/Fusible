use std::path::{Path, PathBuf};

use super::live::{compare_board, sync_positions};

pub(super) fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fusible_live_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn component(name: &str, x: f64) -> String {
    format!(
        r#"{{ "type": "source_component", "source_component_id": "sc_{name}", "name": "{name}" }},
        {{ "type": "pcb_component", "source_component_id": "sc_{name}", "center": {{ "x": {x}, "y": 0 }},
          "rotation": 0, "layer": "top" }},
        {{ "type": "source_port", "source_component_id": "sc_{name}", "pin_number": 1,
          "subcircuit_connectivity_map_key": "net_{name}" }}"#
    )
}

pub(super) fn circuit(names: &[&str]) -> String {
    let parts: Vec<String> = names
        .iter()
        .enumerate()
        .map(|(i, n)| component(n, i as f64 * 3.0))
        .collect();
    format!("[{}]", parts.join(","))
}

fn footprint(name: &str, x: f64, net: &str) -> String {
    format!(
        r#"(footprint "R" (layer "F.Cu") (at {x} 40) (property "Reference" "{name}")
            (pad "1" smd rect (net 1 "{net}")))"#
    )
}

pub(super) fn write_board(dir: &Path, parts: &[(&str, f64, &str)]) -> PathBuf {
    let body: Vec<String> = parts
        .iter()
        .map(|(n, x, net)| footprint(n, *x, net))
        .collect();
    let path = dir.join("board.kicad_pcb");
    std::fs::write(&path, format!("(kicad_pcb {})", body.join(" "))).unwrap();
    path
}

#[test]
fn moved_footprint_is_written_and_the_previous_edits_are_backed_up() {
    let dir = scratch("moved");
    let pcb = write_board(
        &dir,
        &[("R1", 100.0, "A"), ("R2", 103.0, "B"), ("R3", 130.0, "C")],
    );
    let edits = dir.join("manual-edits.json");
    std::fs::write(&edits, r#"{ "pcb_placements": [], "edit_events": [] }"#).unwrap();
    let comparison = compare_board(&pcb, &circuit(&["R1", "R2", "R3"])).unwrap();
    let backups = dir.join(".fusible/backup");
    let moved = sync_positions(&comparison, &edits, &backups, "main").unwrap();
    assert_eq!(moved, ["R3"]);
    assert!(std::fs::read_to_string(&edits).unwrap().contains("R3"));
    assert_eq!(std::fs::read_dir(&backups).unwrap().count(), 1);
    assert!(
        comparison.differences.is_empty(),
        "{:?}",
        comparison.differences
    );
}

#[test]
fn invalid_circuit_json_is_an_error_not_an_empty_circuit() {
    let dir = scratch("invalid");
    let pcb = write_board(&dir, &[("R1", 100.0, "A")]);
    let truncated = r#"[{"type": "source_component", "name": "R1""#;
    let error = compare_board(&pcb, truncated).err().unwrap();
    assert!(error.contains("invalid Circuit JSON"), "{error}");
}
