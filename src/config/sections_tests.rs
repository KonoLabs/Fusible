//! Unit tests for named boards, firmware, and bodies and their cross-references.

use std::path::PathBuf;

use super::*;

const MULTI: &str = r#"
name: rover
boards:
  main: { entry: boards/main/index.tsx, mcu: esp32-s3 }
  sensor: { entry: boards/sensor/index.tsx, mcu: rp2040 }
firmware:
  main-fw: { board: main, pins_rs: fw/main/pins.rs }
  sensor-fw: { board: sensor, pins_rs: fw/sensor/pins.rs }
bodies:
  case: { source: bodies/case.step, build: python bodies/case.py }
  lid: { generate: enclosure, board: sensor, wall_thickness: 1.5 }
"#;

fn invalid(yaml: &str) -> String {
    match FusibleConfig::from_str(yaml) {
        Err(ConfigError::Invalid(message)) => message,
        other => panic!("expected ConfigError::Invalid, got {other:?}"),
    }
}

#[test]
fn parses_named_boards_firmware_and_bodies() {
    let cfg = FusibleConfig::from_str(MULTI).expect("multi-board manifest should parse");
    assert_eq!(cfg.boards.len(), 2);
    assert_eq!(cfg.boards["sensor"].mcu.as_deref(), Some("rp2040"));
    let sensor_fw: Vec<_> = cfg
        .firmware_for(Some("sensor"))
        .map(|(n, _)| n.as_str())
        .collect();
    assert_eq!(sensor_fw, ["sensor-fw"]);
    assert_eq!(
        cfg.bodies["case"].source,
        Some(PathBuf::from("bodies/case.step"))
    );
    assert_eq!(cfg.bodies["lid"].enclosure.wall_thickness, Some(1.5));
    assert_eq!(cfg.board_title("sensor"), "sensor");
}

#[test]
fn rejects_dangling_and_ambiguous_references() {
    assert!(invalid("boards: {a: {}}\nfirmware: {fw: {board: b}}").contains("unknown board `b`"));
    assert!(invalid("boards: {a: {}, b: {}}\nfirmware: {fw: {}}").contains("must name its board"));
    assert!(
        invalid("boards: {a: {}, b: {}}\nbodies: {lid: {generate: enclosure}}")
            .contains("must name its board")
    );
    assert!(invalid("boards: {a: {}}\nbodies: {a: {source: a.step}}").contains("used twice"));
}

#[test]
fn rejects_bodies_that_are_neither_or_both_kinds() {
    assert!(invalid("bodies: {x: {}}").contains("needs a `source`"));
    assert!(invalid("bodies: {x: {source: x.step, generate: enclosure}}").contains("both"));
    assert!(invalid("bodies: {x: {generate: gear}}").contains("unknown generator `gear`"));
    assert!(invalid("bodies: {x: {generate: enclosure, build: make}}").contains("`build`"));
}

#[test]
fn rejects_mixing_legacy_and_named_sections() {
    assert!(invalid("hardware: {}\nboards: {a: {}}").contains("either `boards:`"));
    assert!(invalid("mechanical: {}\nbodies: {x: {source: x.step}}").contains("either `bodies:`"));
}

#[test]
fn legacy_mechanical_section_keeps_its_output_names() {
    let yaml = "name: fieldcam\nmechanical:\n  wall-thickness: 3.0\n";
    let config = FusibleConfig::from_str(yaml).expect("legacy manifest should parse");
    let body = &config.bodies["fieldcam"];
    assert_eq!(body.generate.as_deref(), Some("enclosure"));
    assert_eq!(body.enclosure.wall_thickness, Some(3.0));
    let legacy_dir = Some(PathBuf::from("dist/mechanical"));
    assert_eq!(body.enclosure.out_dir, legacy_dir);

    let clash = FusibleConfig::from_str("name: main\nhardware: {}\nmechanical: {}\n").unwrap();
    assert!(clash.bodies.contains_key("enclosure"));
}

#[test]
fn bodies_name_their_board_with_board_or_the_around_alias() {
    let yaml = concat!(
        "boards: {a: {}, b: {}}\n",
        "bodies:\n",
        "  x: {source: x.step, board: b}\n",
        "  y: {generate: enclosure, around: a}\n",
    );
    let cfg = FusibleConfig::from_str(yaml).expect("board and around both parse");
    assert_eq!(cfg.bodies["x"].board.as_deref(), Some("b"));
    assert_eq!(cfg.bodies["y"].board.as_deref(), Some("a"));
    assert!(
        invalid("boards: {a: {}}\nbodies: {x: {source: x.step, board: z}}")
            .contains("unknown board `z`")
    );
}
