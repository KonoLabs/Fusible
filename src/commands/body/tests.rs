use std::path::PathBuf;

use super::settings::{preview_settings, resolve_settings, Settings, Ventilation};
use super::spec::{build_spec, preview_from};
use super::*;

pub(super) fn settings(ventilation: Ventilation) -> Settings {
    resolve_settings(EnclosureConfig::default(), EnclosureConfig::default())
        .map(|s| Settings { ventilation, ..s })
        .unwrap()
}

#[test]
fn cli_overrides_beat_manifest_and_defaults_fill_the_rest() {
    let s = resolve_settings(
        EnclosureConfig {
            format: Some("all".into()),
            cutouts: Some(false),
            ..Default::default()
        },
        EnclosureConfig {
            format: Some("stl".into()),
            wall_thickness: Some(3.0),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(s.formats, FORMATS);
    assert_eq!(s.wall, 3.0);
    assert!(!s.cutouts);
    assert!(s.mounting);
    assert_eq!(s.out_dir, PathBuf::from("dist/bodies"));
}

#[test]
fn rejects_iges_and_unknown_vents() {
    let bad = |format: &str, vent: &str| {
        resolve_settings(
            EnclosureConfig {
                format: Some(format.into()),
                ventilation: Some(vent.into()),
                ..Default::default()
            },
            EnclosureConfig::default(),
        )
    };
    assert!(matches!(
        bad("iges", "none"),
        Err(BodyError::UnknownFormat { .. })
    ));
    assert!(matches!(
        bad("step", "honeycomb"),
        Err(BodyError::UnknownVentilation { .. })
    ));
}

#[test]
fn bosses_raise_the_board_and_use_pilot_holes() {
    let model = PcbModel::parse(
        r#"[{"type":"pcb_board","width":80,"height":60,"thickness":1.4},
            {"type":"pcb_hole","x":-36,"y":26,"hole_diameter":3.2}]"#,
    );
    let spec = build_spec(&model, &settings(Ventilation::None), Vec::new()).unwrap();
    assert_eq!(spec.outline.len(), 4);
    assert_eq!(spec.board_z, DEFAULT_WALL + STANDOFF);
    assert_eq!(spec.bosses[0].outer, 7.2);
    assert!((spec.bosses[0].pilot - 2.5).abs() < 1e-9);
    assert!((spec.height - (DEFAULT_WALL + STANDOFF + 1.4 + HEADROOM)).abs() < 1e-9);
}

#[test]
fn height_must_clear_the_board_stack() {
    let model = PcbModel::parse(r#"[{"type":"pcb_board","width":20,"height":20}]"#);
    let s = Settings {
        height: Some(3.0),
        ..settings(Ventilation::None)
    };
    assert!(matches!(
        build_spec(&model, &s, Vec::new()),
        Err(BodyError::InvalidDimension { .. })
    ));
    assert!(matches!(
        build_spec(&PcbModel::parse("[]"), &s, Vec::new()),
        Err(BodyError::NoBoard)
    ));
}

#[test]
fn preview_forces_glb_and_reports_the_outer_box() {
    let manifest = EnclosureConfig {
        format: Some("step".into()),
        out_dir: Some(PathBuf::from("elsewhere")),
        wall_thickness: Some(3.0),
        ..Default::default()
    };
    let s = preview_settings(manifest).unwrap();
    assert_eq!(s.formats, vec!["glb"]);
    assert_eq!(s.wall, 3.0);
    let model = PcbModel::parse(
        r#"[{"type":"pcb_board","width":40,"height":154,"thickness":1.4},
            {"type":"pcb_hole","x":0,"y":0,"hole_diameter":3.2}]"#,
    );
    let p = preview_from(&build_spec(&model, &s, Vec::new()).unwrap(), vec![1, 2]);
    let height = 3.0 + STANDOFF + 1.4 + HEADROOM;
    assert_eq!(p.glb, vec![1, 2]);
    assert_eq!(p.board_z, 3.0 + STANDOFF);
    assert!((p.height - height).abs() < 1e-9);
    let expected = [40.0 + 7.0, 154.0 + 7.0, height + 3.0];
    for (got, want) in p.size.iter().zip(expected) {
        assert!((got - want).abs() < 1e-9, "{:?}", p.size);
    }
}

#[test]
fn preview_uses_the_body_of_the_viewed_board() {
    let yaml = concat!(
        "boards: {a: {}, b: {}}\n",
        "bodies:\n",
        "  lid-a: {generate: enclosure, board: a, wall_thickness: 1.0}\n",
        "  lid-b: {generate: enclosure, board: b, wall_thickness: 3.0}\n",
        "  knob: {source: knob.step}\n",
    );
    let cfg = crate::config::FusibleConfig::from_str(yaml).unwrap();
    let wall = |board| fitted_settings(cfg.clone(), board).and_then(|e| e.wall_thickness);
    assert_eq!(wall(Some("a")), Some(1.0));
    assert_eq!(wall(Some("b")), Some(3.0));
    assert_eq!(wall(Some("c")), None);
}
