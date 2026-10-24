use super::*;
use crate::config::error::ConfigError;

const BASE: &str = "\
boards: {main: {}, sensor: {}}
bodies:
  case: {generate: enclosure, board: main}
  bracket: {source: bracket.step}
";

fn invalid(yaml: &str) -> String {
    match FusibleConfig::from_str(yaml) {
        Err(ConfigError::Invalid(message)) => message,
        other => panic!("expected ConfigError::Invalid, got {other:?}"),
    }
}

#[test]
fn parses_an_assembly() {
    let cfg = FusibleConfig::from_str(&format!(
        "{BASE}assembly:
  format: all
  parts:
    main: {{}}
    sensor: {{at: [0, 0, 12], rotate: [0, 0, 90]}}
    case: {{}}
    bracket: {{at: [1, 2, 3]}}"
    ))
    .expect("valid assembly");
    let assembly = cfg.assembly.expect("assembly section");
    assert_eq!(assembly.format.as_deref(), Some("all"));
    assert_eq!(assembly.parts.len(), 4);
    assert_eq!(assembly.parts["sensor"].rotate, Some([0.0, 0.0, 90.0]));
    assert_eq!(assembly.parts["main"], PlacementConfig::default());
}

#[test]
fn rejects_parts_that_cannot_be_placed() {
    let with = |parts: &str| invalid(&format!("{BASE}assembly:\n  parts:\n{parts}"));
    assert!(with("    nope: {}").contains("not a board or a body"));
    assert!(with("    bracket: {}").contains("body without a board"));
    assert!(with("    case: {}").contains("board `main`, which is not in the assembly"));
    assert!(invalid(&format!("{BASE}assembly:\n  parts: {{}}")).contains("no parts"));
    assert!(FusibleConfig::from_str(&format!(
        "{BASE}assembly:\n  parts:\n    main: {{at: [1, 2]}}"
    ))
    .is_err());
}

#[test]
fn a_generated_body_of_the_only_board_follows_it() {
    let cfg = FusibleConfig::from_str(
        "boards: {main: {}}\nbodies: {case: {generate: enclosure}}\nassembly: {parts: {main: {}, case: {}}}",
    )
    .expect("valid assembly");
    assert_eq!(cfg.body_board(&cfg.bodies["case"]), Some("main"));
}
