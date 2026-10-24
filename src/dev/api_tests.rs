use std::path::Path;

use super::project::project_json;
use super::run_request::parse;
use super::server::query_param;
use super::state::State;
use crate::commands::target::BoardTarget;
use crate::config::FusibleConfig;

fn manifest() -> FusibleConfig {
    let yaml = concat!(
        "name: rover\n",
        "boards: {main: {}, sensor: {}}\n",
        "firmware: {main-fw: {board: main, build_cmd: make}, sensor-fw: {board: sensor}}\n",
        "bodies: {case: {source: bodies/case.step}, lid: {generate: enclosure, board: sensor}}\n",
    );
    FusibleConfig::from_str(yaml).unwrap()
}

fn board(name: &str) -> BoardTarget {
    BoardTarget {
        name: Some(name.into()),
        entry: Path::new("/p/boards").join(name).join("c.json"),
        ..Default::default()
    }
}

fn names(items: &serde_json::Value) -> Vec<serde_json::Value> {
    items
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["name"].clone())
        .collect()
}

#[test]
fn query_params_are_decoded() {
    assert_eq!(
        query_param("/x?board=a%20b&rev=1", "board").as_deref(),
        Some("a b")
    );
    assert_eq!(
        query_param("/x?rev=1&board=main", "board").as_deref(),
        Some("main")
    );
    assert_eq!(query_param("/x?boardx=main", "board"), None);
    assert_eq!(query_param("/x", "board"), None);
    assert_eq!(query_param("/x?board=&rev=1", "board"), None);
}

#[test]
fn runs_only_accept_manifest_names_of_the_right_kind() {
    let cfg = manifest();
    let spec = parse(r#"{"action":"build","target":"sensor-fw"}"#, Some(&cfg)).unwrap();
    assert_eq!(spec.label, "build sensor-fw");
    assert_eq!(spec.args, ["build", "--", "sensor-fw"]);
    assert_eq!(
        parse(r#"{"action":"check"}"#, Some(&cfg)).unwrap().args,
        ["check"]
    );
    assert!(parse(r#"{"action":"body","target":"case"}"#, Some(&cfg)).is_ok());
    for bad in [
        r#"{"action":"check","target":"main-fw"}"#,
        r#"{"action":"build","target":"--help"}"#,
        r#"{"action":"export"}"#,
        "not json",
    ] {
        assert!(parse(bad, Some(&cfg)).is_err(), "{bad}");
    }
    assert!(parse(r#"{"action":"check","target":"main"}"#, None).is_err());
}

#[test]
fn project_lists_served_items_with_relative_paths() {
    let state = State::new(&[board("main")]);
    let text = project_json(&state, Some(&manifest()), Path::new("/p"));
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["name"], "rover");
    assert_eq!(json["boards"][0]["entry"], "boards/main/c.json");
    assert_eq!(names(&json["firmware"]), ["main-fw"]);
    assert_eq!(names(&json["bodies"]), ["case"]);
    assert_eq!(json["firmware"][0]["buildable"], true);
}

#[test]
fn viewer_loads_the_project_script_the_server_serves() {
    let page = super::assets::static_response("/", None, false).unwrap();
    assert_eq!(page.status_code().0, 200);
    let script = super::assets::static_response("/project.js", None, false).unwrap();
    assert_eq!(script.status_code().0, 200);
    let html = include_str!("../assets/viewer.html");
    assert!(html.contains(r#"<script src="/project.js"></script>"#));
}
