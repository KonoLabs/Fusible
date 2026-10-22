//! Unit tests for silkscreen text strokes.

use super::super::*;
use super::board_with;

fn layer(extra: &str, name: &str) -> String {
    generate_gerber_files(&board_with(extra), "t").expect("valid circuit")[name].clone()
}

fn silk_text(text: &str, extra: &str) -> String {
    format!(
        r#", {{ "type": "pcb_silkscreen_text", "text": "{text}", "font_size": 1.2,
              "anchor_position": {{ "x": 5, "y": 5 }}, {extra} "layer": "top" }}"#
    )
}

#[test]
fn silkscreen_text_is_stroked() {
    let silk = layer(
        &silk_text("A1", r#""anchor_alignment": "bottom_left","#),
        "F_Silk.gbr",
    );
    assert!(silk.contains("%ADD10C,0.144000*%"), "{silk}");
    assert!(silk.contains("X5000000Y5000000D02*"), "{silk}");
    assert!(silk.contains("D01*"));
}

#[test]
fn silkscreen_text_honors_rotation_and_side() {
    let rotated = layer(
        &silk_text(
            "I",
            r#""anchor_alignment": "bottom_left", "ccw_rotation": 90,"#,
        ),
        "F_Silk.gbr",
    );
    assert!(
        rotated.contains("X3800000Y5400000D02*\nX5000000Y5400000D01*"),
        "{rotated}"
    );
    let bottom = layer(
        &silk_text("I", r#""anchor_alignment": "bottom_left","#).replace("top", "bottom"),
        "B_Silk.gbr",
    );
    assert!(bottom.contains("D01*"));
    let top_silk = layer(
        &silk_text("I", r#""anchor_alignment": "bottom_left","#).replace("top", "bottom"),
        "F_Silk.gbr",
    );
    assert!(!top_silk.contains("D01*"));
}

#[test]
fn silkscreen_text_without_a_glyph_is_an_error() {
    for text in ["\u{3078}", "A#"] {
        assert!(
            generate_gerber_files(&board_with(&silk_text(text, "")), "t").is_err(),
            "{text}"
        );
    }
    let bad_alignment = silk_text("A", r#""anchor_alignment": "middle","#);
    assert!(generate_gerber_files(&board_with(&bad_alignment), "t").is_err());
}
