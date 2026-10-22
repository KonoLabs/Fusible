//! Unit tests for rounded-rect pads, polygon pads and silkscreen text.

use super::super::*;
use super::board_with;

fn layer(extra: &str, name: &str) -> String {
    generate_gerber_files(&board_with(extra), "t").expect("valid circuit")[name].clone()
}

#[test]
fn rounded_rect_pad_uses_a_macro_aperture() {
    let top = layer(
        r#", { "type": "pcb_smtpad", "shape": "rect", "layer": "top", "x": 1, "y": 2,
              "width": 3, "height": 2, "rect_border_radius": 0.5 }"#,
        "F_Cu.gbr",
    );
    let expected = "%AMRR10*\n\
        21,1,3.000000,1.000000,0,0,0*\n\
        21,1,2.000000,2.000000,0,0,0*\n\
        1,1,1.000000,1.000000,0.500000*\n\
        1,1,1.000000,-1.000000,0.500000*\n\
        1,1,1.000000,-1.000000,-0.500000*\n\
        1,1,1.000000,1.000000,-0.500000*\n\
        %\n%ADD10RR10*%\n";
    assert!(top.contains(expected), "{top}");
    assert!(top.contains("X1000000Y2000000D03*"));
}

#[test]
fn plain_rect_pad_stays_a_rectangle_aperture() {
    let top = layer(
        r#", { "type": "pcb_smtpad", "shape": "rect", "layer": "top", "x": 0, "y": 0,
              "width": 3, "height": 2 }"#,
        "F_Cu.gbr",
    );
    assert!(top.contains("%ADD10R,3.000000X2.000000*%"));
    assert!(!top.contains("%AM"));
}

#[test]
fn corner_radius_alias_and_full_radius_are_handled() {
    let alias = layer(
        r#", { "type": "pcb_smtpad", "shape": "rect", "layer": "top", "x": 0, "y": 0,
              "width": 3, "height": 2, "corner_radius": 0.25 }"#,
        "F_Cu.gbr",
    );
    assert!(alias.contains("%AMRR10*"));
    let pill = layer(
        r#", { "type": "pcb_smtpad", "shape": "rect", "layer": "top", "x": 0, "y": 0,
              "width": 3, "height": 2, "rect_border_radius": 1 }"#,
        "F_Cu.gbr",
    );
    assert!(pill.contains("%ADD10O,3.000000X2.000000*%"));
}

#[test]
fn rotated_rounded_rect_becomes_a_region() {
    let top = layer(
        r#", { "type": "pcb_smtpad", "shape": "rotated_rect", "layer": "top", "x": 0, "y": 0,
              "width": 3, "height": 2, "rect_border_radius": 0.5, "ccw_rotation": 30 }"#,
        "F_Cu.gbr",
    );
    assert!(top.contains("G36*") && top.contains("G37*"));
}

#[test]
fn polygon_pad_is_a_closed_region() {
    let top = layer(
        r#", { "type": "pcb_smtpad", "shape": "polygon", "layer": "top",
              "points": [{ "x": 0, "y": 0 }, { "x": 2, "y": 0 }, { "x": 1, "y": 1 }] }"#,
        "F_Cu.gbr",
    );
    let expected = "G36*\nX0Y0D02*\nX2000000Y0D01*\nX1000000Y1000000D01*\nX0Y0D01*\nG37*\n";
    assert!(top.contains(expected), "{top}");
}

#[test]
fn polygon_pad_grows_in_the_solder_mask() {
    let mask = layer(
        r#", { "type": "pcb_smtpad", "shape": "polygon", "layer": "top",
              "points": [{ "x": 0, "y": 0 }, { "x": 2, "y": 0 }, { "x": 2, "y": 2 }, { "x": 0, "y": 2 }] }"#,
        "F_Mask.gbr",
    );
    assert!(mask.contains("X-50000Y-50000D02*"), "{mask}");
    assert!(mask.contains("X2050000Y2050000D01*"), "{mask}");
}

#[test]
fn degenerate_polygon_pads_are_rejected() {
    for points in [
        "[]",
        r#"[{ "x": 0, "y": 0 }, { "x": 1, "y": 0 }]"#,
        r#"[{ "x": 0, "y": 0 }, { "x": 0, "y": 0 }, { "x": 1, "y": 1 }]"#,
    ] {
        let extra = format!(
            r#", {{ "type": "pcb_smtpad", "shape": "polygon", "layer": "top", "points": {points} }}"#
        );
        assert!(
            generate_gerber_files(&board_with(&extra), "t").is_err(),
            "{points}"
        );
    }
}
