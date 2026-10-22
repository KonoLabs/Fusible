//! Unit tests for fail-closed validation, NPTH drills and multilayer stacks.

use super::super::*;
use super::board_with;

#[test]
fn unknown_shapes_fail_instead_of_defaulting() {
    for extra in [
        r#", { "type": "pcb_smtpad", "shape": "polygon", "layer": "top", "x": 0, "y": 0, "points": [] }"#,
        r#", { "type": "pcb_smtpad", "layer": "top", "x": 0, "y": 0, "width": 1, "height": 1 }"#,
        r#", { "type": "pcb_smtpad", "shape": "rect", "layer": "top", "x": 0, "y": 0 }"#,
        r#", { "type": "pcb_plated_hole", "shape": "hole_with_polygon_pad", "x": 0, "y": 0 }"#,
        r#", { "type": "pcb_hole", "hole_shape": "square", "x": 0, "y": 0, "hole_diameter": 1 }"#,
    ] {
        assert!(
            generate_gerber_files(&board_with(extra), "t").is_err(),
            "{extra}"
        );
    }
}

#[test]
fn strict_parse_rejects_bad_input() {
    assert!(matches!(
        PcbModel::try_parse("not json"),
        Err(GerberError::InvalidJson(_))
    ));
    assert!(matches!(
        PcbModel::try_parse("[]"),
        Err(GerberError::NoBoard)
    ));
    assert!(matches!(
        PcbModel::try_parse(&board_with(
            r#", { "type": "pcb_smtpad", "x": "oops", "y": 0 }"#
        )),
        Err(GerberError::Malformed { index: 1, .. })
    ));
    assert!(generate_gerber_files(r#"[{ "type": "pcb_board" }]"#, "t").is_err());
    let no_layer = board_with(
        r#", { "type": "pcb_trace", "route": [
            { "route_type": "wire", "x": 0, "y": 0, "width": 0.2 },
            { "route_type": "wire", "x": 1, "y": 0, "width": 0.2 }
        ] }"#,
    );
    assert!(generate_gerber_files(&no_layer, "t").is_err());
    assert!(generate_gerber_files("[]", "t").is_err());
    assert_eq!(PcbModel::parse("not json").boards.len(), 0);
}

#[test]
fn mounting_holes_go_to_npth_drill_file() {
    let json = board_with(
        r#", { "type": "pcb_hole", "hole_shape": "circle", "x": 3, "y": 4, "hole_diameter": 3.2 }"#,
    );
    let files = generate_gerber_files(&json, "t").unwrap();
    let npth = &files["drill_npth.drl"];
    assert!(npth.contains("TF.FileFunction,NonPlated,1,2,NPTH"));
    assert!(npth.contains("T01C3.2000\n"));
    assert!(npth.contains("X3.0000Y4.0000"));
    assert!(!files["drill_pth.drl"].contains("X3.0000Y4.0000"));
}

#[test]
fn zero_width_silkscreen_is_skipped_but_negative_fails() {
    let path = |w: &str| {
        board_with(&format!(
            r#", {{ "type": "pcb_silkscreen_path", "layer": "top", "stroke_width": {w},
                "route": [{{ "x": 0, "y": 0 }}, {{ "x": 1, "y": 0 }}] }}"#
        ))
    };
    let silk = generate_gerber_files(&path("0"), "t").unwrap();
    assert!(!silk["F_Silk.gbr"].contains("D01*"));
    assert!(generate_gerber_files(&path("-0.1"), "t").is_err());
}

#[test]
fn test_mounting_holes_get_their_own_npth_drill_file() {
    let with_hole = r#"[
        { "type": "pcb_board", "width": 20.0, "height": 20.0 },
        { "type": "pcb_hole", "hole_shape": "circle", "x": 5.0, "y": -5.0, "hole_diameter": 3.2 }
    ]"#;
    let files = generate_gerber_files(with_hole, "TestBoard").unwrap();
    let npth = &files["drill_npth.drl"];
    assert!(npth.contains("NonPlated,1,2,NPTH"));
    assert!(npth.contains("C3.2000"));
    assert!(npth.contains("X5.0000Y-5.0000"));
    assert!(!files["drill_pth.drl"].contains("C3.2000"));

    let without = generate_gerber_files(
        r#"[{ "type": "pcb_board", "width": 20.0, "height": 20.0 }]"#,
        "TestBoard",
    )
    .unwrap();
    assert!(!without.contains_key("drill_npth.drl"));
}

#[test]
fn four_layer_boards_get_inner_copper_files() {
    let board = |inner: &str| {
        format!(
            r#"[
            {{ "type": "pcb_board", "width": 20.0, "height": 20.0, "num_layers": 4 }},
            {{ "type": "pcb_trace", "route": [
                {{ "route_type": "wire", "x": 0.0, "y": 0.0, "width": 0.2, "layer": "{inner}" }},
                {{ "route_type": "wire", "x": 5.0, "y": 0.0, "width": 0.2, "layer": "{inner}" }}
            ] }}
        ]"#
        )
    };
    let files = generate_gerber_files(&board("inner2"), "t").unwrap();
    assert!(files["In1_Cu.gbr"].contains("Copper,L2,Inr"));
    assert!(files["In2_Cu.gbr"].contains("Copper,L3,Inr"));
    assert!(files["In2_Cu.gbr"].contains("D01*"));
    assert!(!files["In1_Cu.gbr"].contains("D01*"));
    assert!(files["B_Cu.gbr"].contains("Copper,L4,Bot"));
    assert!(generate_gerber_files(&board("inner3"), "t").is_err());
}
