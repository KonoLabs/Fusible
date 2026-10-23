use super::eject_manifest::eject_manifest;
use crate::config::FusibleConfig;

const PATH: &str = "bodies/lid.step";

fn authored(text: &str, name: &str) -> (Option<String>, Option<String>) {
    let body = &FusibleConfig::from_str(text).unwrap().bodies[name];
    let source = body.source.as_ref().map(|p| p.display().to_string());
    (source, body.generate.clone())
}

const BLOCK: &str = "\
name: rover  # the project
boards:
  main: { entry: boards/main.json }
  sensor: { entry: boards/sensor.json }

# physical parts
bodies:
  case:
    source: bodies/case.step
  lid:
    # generated for the sensor
    generate: enclosure
    board: sensor
    wall_thickness: 1.5
    format: all
    ventilation: grid  # airflow

  knob: { source: bodies/knob.step }
firmware:
  fw: { board: main }
";

#[test]
fn block_entry_becomes_authored_and_keeps_comments() {
    let out = eject_manifest(BLOCK, "lid", PATH).unwrap();
    assert_eq!(authored(&out, "lid"), (Some(PATH.into()), None));
    for kept in [
        "name: rover  # the project",
        "# physical parts",
        "    # generated for the sensor",
        "    board: sensor",
        "  knob: { source: bodies/knob.step }",
        "firmware:",
    ] {
        assert!(out.contains(kept), "lost `{kept}` in:\n{out}");
    }
    assert!(!out.contains("wall_thickness") && !out.contains("airflow"));
    assert_eq!(out.matches("source:").count(), 3);
    let sensor = FusibleConfig::from_str(&out).unwrap().bodies["lid"]
        .board
        .clone();
    assert_eq!(sensor.as_deref(), Some("sensor"));
}

#[test]
fn flow_entry_becomes_authored_with_its_board_and_comment() {
    let text = concat!(
        "boards: {a: {}, b: {}}\n",
        "bodies:\n",
        "  lid: { generate: enclosure, board: b, wall_thickness: 1.5 }  # keep me\n",
        "  x: { source: x.step }\n",
    );
    let out = eject_manifest(text, "lid", PATH).unwrap();
    assert!(
        out.contains("  lid: { source: bodies/lid.step, board: b }  # keep me"),
        "{out}"
    );
    assert_eq!(authored(&out, "lid"), (Some(PATH.into()), None));
    assert!(out.contains("  x: { source: x.step }"));
}

#[test]
fn legacy_mechanical_block_moves_into_a_bodies_section() {
    let text = concat!(
        "name: fieldcam\nhardware:\n  entry: hardware/index.tsx\n\n",
        "mechanical:\n  format: all\n  wall_thickness: 2.5\n\n",
        "firmware:\n  path: apps/firmware\n",
    );
    let out = eject_manifest(text, "fieldcam", "bodies/enclosure.step").unwrap();
    assert!(!out.contains("mechanical:") && !out.contains("wall_thickness"));
    assert!(!out.contains("\n\n\n"), "{out}");
    assert!(
        out.ends_with("\nbodies:\n  fieldcam:\n    source: bodies/enclosure.step\n"),
        "{out}"
    );
    assert_eq!(
        authored(&out, "fieldcam").0.as_deref(),
        Some("bodies/enclosure.step")
    );
    assert!(out.contains("firmware:\n  path: apps/firmware"));
}

#[test]
fn legacy_one_line_mechanical_is_replaced() {
    let text = "name: p\nmechanical: { height: 20 }\n";
    let out = eject_manifest(text, "p", PATH).unwrap();
    assert_eq!(
        out,
        "name: p\n\nbodies:\n  p:\n    source: bodies/lid.step\n"
    );
}

#[test]
fn odd_paths_are_quoted() {
    let out = eject_manifest(BLOCK, "lid", "my parts/lid, v2.step").unwrap();
    assert!(
        out.contains("    source: \"my parts/lid, v2.step\""),
        "{out}"
    );
    assert_eq!(
        authored(&out, "lid").0.as_deref(),
        Some("my parts/lid, v2.step")
    );
}

#[test]
fn unsafe_shapes_are_refused_and_nothing_is_returned() {
    let multi = "bodies:\n  lid:\n    generate: enclosure\n    wall_thickness:\n      3.0\n";
    let quoted = "bodies:\n  lid: { generate: \"enclosure\" }\n";
    let nested = "bodies:\n  lid: { generate: enclosure,\n    height: 20 }\n";
    let crlf = "bodies:\r\n  lid:\r\n    generate: enclosure\r\n";
    let missing = "boards: {a: {}}\n";
    let no_entry = "bodies:\n  other: { source: o.step }\n";
    for (text, why) in [
        (multi, "several lines"),
        (quoted, "quotes"),
        (nested, "several lines"),
        (crlf, "CRLF"),
        (missing, "neither"),
        (no_entry, "not in"),
    ] {
        let err = eject_manifest(text, "lid", PATH).unwrap_err();
        assert!(err.contains(why), "`{err}` should mention `{why}`");
    }
}

#[test]
fn an_authored_body_cannot_be_ejected_again() {
    let err = eject_manifest(BLOCK, "case", PATH).unwrap_err();
    assert!(err.contains("already has a `source:`"), "{err}");
}
