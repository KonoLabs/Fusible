use std::path::Path;

use super::project::project_json;
use super::run_request::parse;
use super::state::State;
use crate::commands::target::BoardTarget;
use crate::config::FusibleConfig;

fn manifest(assembly: bool) -> FusibleConfig {
    let tail = if assembly {
        "assembly: {parts: {main: {}}}\n"
    } else {
        ""
    };
    FusibleConfig::from_str(&format!("boards: {{main: {{}}}}\n{tail}")).unwrap()
}

#[test]
fn assembly_runs_take_no_board_target() {
    let cfg = manifest(true);
    for body in [
        r#"{"action":"assembly"}"#,
        r#"{"action":"assembly","target":"assembly"}"#,
    ] {
        let spec = parse(body, Some(&cfg)).unwrap();
        assert_eq!(spec.label, "assembly");
        assert_eq!(spec.args, ["assembly"]);
    }
    assert!(parse(r#"{"action":"assembly","target":"main"}"#, Some(&cfg)).is_err());
    assert!(parse(r#"{"action":"assembly","target":"--help"}"#, None).is_err());
}

#[test]
fn project_reports_whether_an_assembly_is_declared() {
    let state = State::new(&[BoardTarget {
        name: Some("main".into()),
        entry: Path::new("/p/c.json").to_path_buf(),
        ..Default::default()
    }]);
    let with =
        FusibleConfig::from_str("boards: {main: {}}\nassembly: {parts: {main: {}}}\n").unwrap();
    let flag = |cfg: Option<&FusibleConfig>| {
        let text = project_json(&state, cfg, Path::new("/p"));
        serde_json::from_str::<serde_json::Value>(&text).unwrap()["assembly"].clone()
    };
    assert_eq!(flag(Some(&with)), true);
    assert_eq!(flag(Some(&manifest(false))), false);
    assert_eq!(flag(None), false);
}
