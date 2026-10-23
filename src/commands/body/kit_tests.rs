use super::kit_data::build_kit;
use super::{BodyError, CLEARANCE, EDGE_TOLERANCE, NOTCH_MARGIN};

const BOARD: &str = r#"[
  {"type":"pcb_board","width":40,"height":30,"center":{"x":0,"y":0},"thickness":1.4},
  {"type":"pcb_hole","x":-15,"y":10,"hole_diameter":3.2},
  {"type":"pcb_plated_hole","x":5,"y":5,"shape":"circle","hole_diameter":1.0,"outer_diameter":1.6},
  {"type":"source_component","source_component_id":"sc1","name":"J1"},
  {"type":"source_component","source_component_id":"sc2","name":"R1"},
  {"type":"pcb_component","pcb_component_id":"pc1","source_component_id":"sc1",
   "center":{"x":18,"y":0},"width":6,"height":8,"layer":"top"},
  {"type":"pcb_component","pcb_component_id":"pc2","source_component_id":"sc2",
   "center":{"x":0,"y":0},"width":2,"height":1,"layer":"bottom"}
]"#;

#[test]
fn kit_lists_outline_holes_components_and_edge_openings() {
    let kit = build_kit(BOARD, Some("main".into())).unwrap();
    assert_eq!(kit.board.as_deref(), Some("main"));
    assert_eq!(kit.thickness, 1.4);
    assert_eq!(kit.outline.len(), 4);
    let xs: Vec<f64> = kit.outline.iter().map(|p| p[0]).collect();
    assert!(xs.iter().all(|x| x.abs() == 20.0), "{xs:?}");

    let holes: Vec<_> = kit
        .holes
        .iter()
        .map(|h| (h.x, h.y, h.diameter, h.plated))
        .collect();
    assert_eq!(holes, [(-15.0, 10.0, 3.2, false), (5.0, 5.0, 1.0, true)]);

    let names: Vec<_> = kit
        .components
        .iter()
        .map(|c| (c.name.as_str(), c.touches_edge))
        .collect();
    assert_eq!(names, [("J1", true), ("R1", false)]);
    assert_eq!(kit.components[1].layer, "bottom");

    assert_eq!(kit.openings.len(), 1);
    let opening = &kit.openings[0];
    assert_eq!((opening.component.as_str(), opening.side), ("J1", "+x"));
    assert!((opening.normal[0] - 1.0).abs() < 1e-9 && opening.normal[1].abs() < 1e-9);
    assert_eq!(opening.from[0], 20.0);
    assert_eq!(opening.to[0], 20.0);
    let mut ys = [opening.from[1], opening.to[1]];
    ys.sort_by(f64::total_cmp);
    assert_eq!(ys, [-4.5, 4.5]);
}

#[test]
fn kit_exports_the_notch_constants_the_generator_uses() {
    let kit = build_kit(BOARD, None).unwrap();
    assert_eq!(kit.notch.margin, NOTCH_MARGIN);
    assert_eq!(kit.notch.edge_tolerance, EDGE_TOLERANCE);
    assert_eq!(kit.notch.clearance, CLEARANCE);
    let value = serde_json::to_value(&kit).unwrap();
    assert_eq!(value["notch"]["margin"], 0.5);
    assert_eq!(value["warnings"], serde_json::json!([]));
}

#[test]
fn holes_without_a_diameter_are_left_out_with_a_warning() {
    let board = r#"[
      {"type":"pcb_board","width":40,"height":30,"center":{"x":0,"y":0}},
      {"type":"pcb_hole","x":1,"y":1,"hole_diameter":3.0},
      {"type":"pcb_hole","x":2,"y":2}
    ]"#;
    let kit = build_kit(board, None).unwrap();
    assert_eq!(kit.holes.len(), 1);
    assert_eq!(kit.holes[0].diameter, 3.0);
    assert_eq!(kit.warnings.len(), 1);
    assert!(kit.warnings[0].contains("1 hole(s)"), "{:?}", kit.warnings);
}

#[test]
fn invalid_circuit_json_is_an_error_not_an_empty_kit() {
    assert!(matches!(
        build_kit("not json", None),
        Err(BodyError::InvalidCircuit(_))
    ));
}

#[test]
fn kit_serializes_the_documented_keys() {
    let kit = build_kit(BOARD, None).unwrap();
    let value = serde_json::to_value(&kit).unwrap();
    for key in [
        "board",
        "units",
        "frame",
        "outline",
        "thickness",
        "holes",
        "components",
        "openings",
        "notch",
        "warnings",
    ] {
        assert!(value.get(key).is_some(), "missing `{key}`");
    }
    assert_eq!(value["units"], "mm");
    assert_eq!(value["openings"][0]["side"], "+x");
}

#[test]
fn kit_without_a_board_outline_is_an_error() {
    assert!(matches!(build_kit("[]", None), Err(BodyError::NoBoard)));
    let two_points = r#"[{"type":"pcb_board","outline":[{"x":0,"y":0},{"x":1,"y":1}]}]"#;
    assert!(matches!(
        build_kit(two_points, None),
        Err(BodyError::NoBoard)
    ));
}
