//! Tests for the hint printed when several chips could be the MCU.

use super::fixtures::chip;
use crate::codegen::select::hint_message;
use crate::parser::{Circuit, Component};
use serde_json::json;

#[test]
fn hint_names_each_candidate_chip_by_part_number() {
    let board = Circuit {
        elements: vec![chip("U1", "U1"), chip("U2", "U2")],
        raw_elements: vec![
            json!({"type": "source_component", "source_component_id": "U1", "manufacturer_part_number": "OV2640"}),
            json!({"type": "source_component", "source_component_id": "U2", "manufacturer_part_number": "ESP32-S3R8"}),
        ],
    };
    let chips: Vec<&Component> = board.components().collect();
    let hint = hint_message(&board, &chips);
    assert!(hint.contains("U1 [OV2640]"), "got: {hint}");
    assert!(hint.contains("U2 [ESP32-S3R8]"), "got: {hint}");
}
