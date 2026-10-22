//! Tests for MCU component selection.

use super::fixtures::chip;
use crate::codegen::select::select_mcu_component;
use crate::parser::{Circuit, CircuitElement, Component};
use serde_json::json;

fn passive(component_id: &str, name: &str, kind: &str) -> CircuitElement {
    CircuitElement::Component(Component {
        component_id: Some(component_id.to_string()),
        kind: Some(kind.to_string()),
        name: Some(name.to_string()),
    })
}

fn part(component_id: &str, mpn: &str) -> serde_json::Value {
    json!({
        "type": "source_component",
        "source_component_id": component_id,
        "manufacturer_part_number": mpn,
    })
}

fn circuit(elements: Vec<CircuitElement>, raw: Vec<serde_json::Value>) -> Circuit {
    Circuit {
        elements,
        raw_elements: raw,
    }
}

fn selected_id(circuit: &Circuit, target: Option<&str>) -> Option<String> {
    select_mcu_component(circuit, target).and_then(|c| c.component_id.clone())
}

#[test]
fn passive_named_mcu_is_never_selected() {
    let board = circuit(
        vec![
            passive("C_MCU", "C_MCU", "simple_capacitor"),
            chip("U10", "U10"),
            chip("U1", "U1"),
        ],
        vec![part("U10", "AP2112K-3.3"), part("U1", "ESP32-S3R8")],
    );
    assert_eq!(selected_id(&board, None).as_deref(), Some("U1"));
}

#[test]
fn target_matches_the_part_number_of_a_chip() {
    let board = circuit(
        vec![chip("U1", "U1"), chip("U2", "U2")],
        vec![part("U1", "OV2640"), part("U2", "ESP32-S3R8")],
    );
    assert_eq!(selected_id(&board, Some("esp32-s3")).as_deref(), Some("U2"));
}

#[test]
fn two_unknown_chips_without_target_select_nothing() {
    let board = circuit(vec![chip("U1", "U1"), chip("U2", "U2")], Vec::new());
    assert!(select_mcu_component(&board, None).is_none());
}

#[test]
fn single_chip_is_selected_even_when_unknown() {
    let board = circuit(
        vec![passive("C1", "C1", "simple_capacitor"), chip("U7", "U7")],
        Vec::new(),
    );
    assert_eq!(selected_id(&board, None).as_deref(), Some("U7"));
}
