use super::live::{compare_board, sync_positions};
use super::live_tests::{circuit, scratch};
use super::tsx_tag::{find_tag, with_number};

#[test]
fn a_turned_part_is_rewritten_only_when_the_tsx_has_a_plain_rotation() {
    let dir = scratch("tsx_turn");
    let pcb = r#"(kicad_pcb
        (footprint "R" (layer "F.Cu") (at 100 40) (property "Reference" "R1"))
        (footprint "R" (layer "F.Cu") (at 103 40 90) (property "Reference" "R2"))
        (footprint "R" (layer "F.Cu") (at 106 40 90) (property "Reference" "R3")))"#;
    std::fs::write(dir.join("board.kicad_pcb"), pcb).unwrap();
    let tsx = [
        r#"<resistor name="R1" pcbX={0} pcbY={0} />"#,
        r#"<resistor name="R2" pcbX={3} pcbY={0} pcbRotation={0} />"#,
        r#"<resistor name="R3" pcbX={6} pcbY={0} />"#,
    ];
    std::fs::write(dir.join("index.tsx"), tsx.join("\n")).unwrap();
    let found = compare_board(&dir.join("board.kicad_pcb"), &circuit(&["R1", "R2", "R3"]));
    let written = sync_positions(&found.unwrap(), &dir, &dir.join("b"), "main").unwrap();
    assert_eq!(written, ["R2"]);
    let text = std::fs::read_to_string(dir.join("index.tsx")).unwrap();
    assert!(text.contains(r#"name="R2" pcbX={3} pcbY={0} pcbRotation={90}"#));
    assert!(text.contains(r#"name="R3" pcbX={6} pcbY={0} />"#));
}

#[test]
fn a_literal_prop_is_replaced_in_the_style_it_was_written() {
    let braces = r#"<resistor name="R1" pcbX={-10} pcbY={ 2.5 } />"#;
    let edited = with_number(braces, "pcbX", 4.25).unwrap();
    assert_eq!(edited, r#"<resistor name="R1" pcbX={4.25} pcbY={ 2.5 } />"#);
    let quoted = r#"<resistor name="R1" pcbX="-10mm" />"#;
    assert_eq!(
        with_number(quoted, "pcbX", 3.0).unwrap(),
        r#"<resistor name="R1" pcbX="3mm" />"#
    );
}

#[test]
fn an_expression_or_a_missing_prop_is_not_rewritten() {
    let tag = r#"<resistor name="R1" pcbX={base + 2} />"#;
    assert_eq!(with_number(tag, "pcbX", 1.0), None);
    assert_eq!(with_number(tag, "pcbY", 1.0), None);
    assert_eq!(with_number(r#"<r xpcbX={1} />"#, "pcbX", 1.0), None);
}

#[test]
fn a_part_is_found_only_when_exactly_one_tag_carries_its_name() {
    let source = concat!(
        "<board>\n  <resistor\n    name=\"R1\"\n    pcbX={1}\n    footprint={\"0603\"}\n  />\n",
        "  <capacitor name=\"C1\" />\n</board>"
    );
    let range = find_tag(source, "R1").unwrap();
    assert!(source[range.clone()].starts_with("<resistor"));
    assert!(source[range].ends_with("/>"));
    assert_eq!(find_tag(source, "R2"), None);
    let twice = format!("{source}{source}");
    assert_eq!(find_tag(&twice, "R1"), None);
}
