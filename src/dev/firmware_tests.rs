use std::path::Path;

use serde_json::Value;

use super::firmware::firmware_reply;
use super::state::State;
use crate::commands::target::BoardTarget;
use crate::config::FusibleConfig;

const ESP32: &str = r#"[
  {"type":"source_component","component_id":"U1","name":"ESP32-S3","kind":"chip"},
  {"type":"source_net","net_id":"n_cam","name":"CAM_D0"},
  {"type":"source_port","port_id":"U1.GPIO4","source_component_id":"U1",
   "name":"GPIO4","net_id":"n_cam","pin_number":4},
  {"type":"source_port","port_id":"CAM1.D0","source_component_id":"CAM1",
   "name":"D0","net_id":"n_cam","pin_number":1},
  {"type":"source_net","net_id":"n_tx","name":"MODEM_TX"},
  {"type":"source_port","port_id":"U1.GPIO43","source_component_id":"U1",
   "name":"GPIO43","net_id":"n_tx","pin_number":43}
]"#;
const NO_MCU: &str = r#"[
  {"type":"source_component","component_id":"R1","name":"R1","kind":"simple_resistor"}
]"#;
const PIN_MUX: &str = include_str!("../../examples/circuit-pinmux-violation.json");

fn manifest(mcu: &str) -> FusibleConfig {
    let yaml = format!(
        "boards: {{main: {{mcu: {mcu}}}}}\n\
         firmware: {{app: {{board: main, framework: embassy, pins_rs: fw/pins.rs}}}}\n"
    );
    FusibleConfig::from_str(&yaml).unwrap()
}

fn state_with(source: Option<&str>) -> State {
    let board = BoardTarget {
        name: Some("main".into()),
        ..Default::default()
    };
    let state = State::new(&[board]);
    state.slots[0].set_source(source);
    state
}

fn reply(cfg: &FusibleConfig, state: &State, name: &str) -> (u16, Value) {
    let (status, body) = firmware_reply(Some(cfg), state, name, Path::new("/p"));
    (status, serde_json::from_str(&body).unwrap())
}

fn pin<'a>(json: &'a Value, label: &str) -> &'a Value {
    json["pins"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["pin"] == label)
        .unwrap_or_else(|| panic!("no pin {label}"))
}

#[test]
fn pin_map_lists_pins_with_groups_nets_and_rust_names() {
    let (status, json) = reply(&manifest("esp32-s3"), &state_with(Some(ESP32)), "app");
    assert_eq!(status, 200);
    assert_eq!(json["board"], "main");
    assert_eq!(json["mcu"], "esp32-s3");
    assert_eq!(json["framework"], "embassy");
    assert_eq!(json["pins_rs"], "fw/pins.rs");
    assert_eq!(json["generated"], false);
    let cam = pin(&json, "GPIO4");
    assert_eq!(cam["net"], "CAM_D0");
    assert_eq!(cam["group"], "Camera");
    assert_eq!(cam["remote"], "CAM1.D0");
    assert_eq!(cam["const"], "gpio::CAM_D0");
    assert_eq!(cam["field"], "CAMERA_PINS.cam_d0");
    assert_eq!(pin(&json, "GPIO43")["group"], "Modem");
    assert!(json["violations"].as_array().unwrap().is_empty());
}

#[test]
fn pin_mux_conflict_is_attached_to_its_pin() {
    let (status, json) = reply(&manifest("stm32g431"), &state_with(Some(PIN_MUX)), "app");
    assert_eq!(status, 200);
    let violation = pin(&json, "PA5")["violation"].as_str().unwrap();
    assert!(violation.contains("PA5"));
    assert_eq!(json["violations"].as_array().unwrap().len(), 1);
}

#[test]
fn missing_mcu_is_a_helpful_error_not_a_crash() {
    let (status, json) = reply(&manifest("esp32-s3"), &state_with(Some(NO_MCU)), "app");
    assert_eq!(status, 422);
    let message = json["error"].as_str().unwrap();
    assert!(message.contains("no MCU"));
    assert!(message.contains("`mcu:` on board `main`"));
}

#[test]
fn unknown_firmware_and_unloaded_boards_are_reported() {
    let cfg = manifest("esp32-s3");
    assert_eq!(reply(&cfg, &state_with(Some(ESP32)), "nope").0, 404);
    assert_eq!(reply(&cfg, &state_with(None), "app").0, 503);
    let (status, _) = firmware_reply(None, &state_with(None), "app", Path::new("/p"));
    assert_eq!(status, 404);
}
