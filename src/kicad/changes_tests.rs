use super::live::{compare_board, sync_positions};
use super::live_tests::{circuit, scratch, write_board};

#[test]
fn a_renamed_part_is_reported_and_not_written() {
    let dir = scratch("renamed");
    let pcb = write_board(
        &dir,
        &[("R1", 100.0, "A"), ("R2", 103.0, "B"), ("R9", 106.0, "C")],
    );
    let edits = dir.join("manual-edits.json");
    let comparison = compare_board(&pcb, &circuit(&["R1", "R2", "R3"])).unwrap();
    let moved = sync_positions(&comparison, &edits, &dir.join("b"), "main").unwrap();
    assert!(moved.is_empty());
    assert!(!edits.exists());
    assert_eq!(comparison.differences, ["R3 renamed to R9 in KiCad"]);
}

#[test]
fn an_added_part_is_reported_and_not_written() {
    let dir = scratch("added");
    let pcb = write_board(
        &dir,
        &[("R1", 100.0, "A"), ("R2", 103.0, "B"), ("R7", 160.0, "C")],
    );
    let edits = dir.join("manual-edits.json");
    let comparison = compare_board(&pcb, &circuit(&["R1", "R2"])).unwrap();
    assert_eq!(comparison.differences, ["R7 added in KiCad"]);
    let moved = sync_positions(&comparison, &edits, &dir.join("b"), "main").unwrap();
    assert!(moved.is_empty());
    assert!(!edits.exists());
}

#[test]
fn a_rewired_net_is_reported() {
    let dir = scratch("nets");
    let two = r#"[
        {"type":"source_component","source_component_id":"a","name":"R1"},
        {"type":"source_component","source_component_id":"b","name":"R2"},
        {"type":"pcb_component","source_component_id":"a","center":{"x":0,"y":0},"rotation":0,"layer":"top"},
        {"type":"pcb_component","source_component_id":"b","center":{"x":3,"y":0},"rotation":0,"layer":"top"},
        {"type":"source_port","source_component_id":"a","pin_number":1,"subcircuit_connectivity_map_key":"n"},
        {"type":"source_port","source_component_id":"b","pin_number":1,"subcircuit_connectivity_map_key":"n"}]"#;
    let same = write_board(&dir, &[("R1", 100.0, "SIG"), ("R2", 103.0, "SIG")]);
    assert!(compare_board(&same, two).unwrap().differences.is_empty());
    let split = write_board(&dir, &[("R1", 100.0, "SIG"), ("R2", 103.0, "OTHER")]);
    let differences = compare_board(&split, two).unwrap().differences;
    assert_eq!(differences, ["net differs, the design connects R1.1 R2.1"]);
}
