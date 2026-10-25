use super::place::IDENTITY;
use super::probe::{probes, Probe};
use super::rules::{Measurement, Response, Violation};
use crate::commands::body::build_kit;

const BOARD: &str = r#"[
  {"type":"pcb_board","width":40,"height":30,"center":{"x":0,"y":0},"thickness":1.4},
  {"type":"pcb_hole","x":-15,"y":10,"hole_diameter":3.2},
  {"type":"source_component","source_component_id":"sc1","name":"J1"},
  {"type":"pcb_component","pcb_component_id":"pc1","source_component_id":"sc1",
   "center":{"x":18,"y":0},"width":6,"height":8,"layer":"top"},
  {"type":"cad_component","pcb_component_id":"pc1","position":{"x":18,"y":0,"z":3.5},"size":{"x":6,"y":8,"z":4.0}}
]"#;

pub fn board_probes() -> Vec<Probe> {
    let kit = build_kit(BOARD, Some("main".into())).unwrap();
    probes("main", &kit, &IDENTITY)
}

pub fn hit(probe: &str, volume: f64) -> Measurement {
    Measurement {
        probe: probe.into(),
        body: "case".into(),
        volume,
        at: Some([1.0, 2.0, 3.0]),
        collision: Some("collision:0".into()),
    }
}

pub fn response(hits: Vec<Measurement>) -> Response {
    Response {
        hits,
        ..Default::default()
    }
}

pub fn rules(found: &[Violation]) -> Vec<&str> {
    found.iter().map(|v| v.rule).collect()
}
