use std::path::Path;

use super::api::circuit_reply;
use super::project::project_json;
use super::state::{Health, State};
use crate::commands::target::BoardTarget;
use crate::config::FusibleConfig;

fn board(name: &str) -> BoardTarget {
    BoardTarget {
        name: Some(name.into()),
        entry: Path::new("/p/boards").join(name).join("c.json"),
        ..Default::default()
    }
}

fn manifest() -> FusibleConfig {
    FusibleConfig::from_str("boards: {main: {}}\n").unwrap()
}

#[test]
fn an_unknown_board_is_a_404_and_a_board_in_progress_is_an_empty_object() {
    let state = State::new(&[board("main")]);
    let (status, body) = circuit_reply(&state, Some("ghost"));
    assert_eq!(status, 404);
    assert!(body.contains("unknown board"));
    assert_eq!(circuit_reply(&state, Some("main")), (200, "{}".to_string()));
    assert_eq!(circuit_reply(&state, None), (200, "{}".to_string()));
    state.slots[0].publish("{\"ok\":1}".into(), Health::default(), None);
    assert_eq!(circuit_reply(&state, Some("main")).1, "{\"ok\":1}");
}

#[test]
fn a_board_is_pending_and_fails_closed_until_its_first_build() {
    let state = State::new(&[board("main")]);
    let text = project_json(&state, Some(&manifest()), Path::new("/p"));
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["boards"][0]["pending"], true);
    assert_eq!(json["boards"][0]["fuse_blown"], true);
    let health = Health {
        fuse_blown: false,
        violations: 0,
    };
    state.slots[0].publish("{}".into(), health, None);
    let text = project_json(&state, Some(&manifest()), Path::new("/p"));
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["boards"][0]["pending"], false);
    assert_eq!(json["boards"][0]["fuse_blown"], false);
}
