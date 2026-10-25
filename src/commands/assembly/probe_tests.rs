use super::place::{apply, placement, IDENTITY};
use super::probe::{probes, Probe, Role, Shape, Side};
use crate::commands::body::build_kit;
use crate::config::PlacementConfig;

pub(super) const BOARD: &str = r#"[
  {"type":"pcb_board","width":40,"height":30,"center":{"x":0,"y":0},"thickness":1.4},
  {"type":"pcb_hole","x":-15,"y":10,"hole_diameter":3.2},
  {"type":"source_component","source_component_id":"sc1","name":"J1"},
  {"type":"pcb_component","pcb_component_id":"pc1","source_component_id":"sc1",
   "center":{"x":18,"y":0},"width":6,"height":8,"layer":"top"},
  {"type":"cad_component","pcb_component_id":"pc1","position":{"x":18,"y":0,"z":3.5},"size":{"x":6,"y":8,"z":4.0}}
]"#;

pub(super) fn near_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

pub(super) fn built(frame: &[f64; 12]) -> Vec<Probe> {
    let kit = build_kit(BOARD, Some("main".into())).unwrap();
    probes("main", &kit, frame)
}

#[test]
fn a_board_with_one_hole_and_one_connector_gets_every_probe_kind() {
    let all = built(&IDENTITY);
    let count = |f: fn(&Role) -> bool| all.iter().filter(|p| f(&p.role)).count();
    assert_eq!(all.len(), 1 + 1 + 1 + 2 * 5 + 1);
    assert_eq!(count(|r| matches!(r, Role::Slab)), 1);
    assert_eq!(count(|r| matches!(r, Role::KeepOut { .. })), 1);
    assert_eq!(count(|r| matches!(r, Role::Opening { .. })), 1);
    assert_eq!(count(|r| matches!(r, Role::HoleNear { .. })), 2);
    assert_eq!(count(|r| matches!(r, Role::HoleQuadrant { .. })), 8);
    let mut ids: Vec<&str> = all.iter().map(|p| p.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), all.len());
    let value = serde_json::to_value(&all[0]).unwrap();
    assert_eq!(value["kind"], "prism");
    assert_eq!(value["show"], true);
    assert_eq!(value["matrix"].as_array().unwrap().len(), 12);
}

#[test]
fn the_slab_shrinks_by_the_inset_and_follows_the_placement() {
    let turn = placement(&PlacementConfig {
        at: Some([10.0, 0.0, 0.0]),
        rotate: Some([0.0, 0.0, 90.0]),
    });
    let all = built(&turn);
    let slab = &all[0];
    let Shape::Prism { points, z0, z1 } = &slab.shape else {
        panic!("the slab is a prism: {:?}", slab.shape);
    };
    assert!(near_eq(*z0, -0.6) && near_eq(*z1, 0.6));
    let mut placed: Vec<[f64; 2]> = points
        .iter()
        .map(|p| {
            let q = apply(&slab.matrix, [p[0], p[1], 0.0]);
            [q[0], q[1]]
        })
        .collect();
    placed.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let expected = [[-4.9, -19.9], [-4.9, 19.9], [24.9, -19.9], [24.9, 19.9]];
    for (got, want) in placed.iter().zip(expected) {
        assert!(
            near_eq(got[0], want[0]) && near_eq(got[1], want[1]),
            "{placed:?}"
        );
    }
}

#[test]
fn the_opening_probe_reaches_outward_from_the_edge_over_the_connector_height() {
    let all = built(&IDENTITY);
    let opening = all
        .iter()
        .find(|p| matches!(p.role, Role::Opening { .. }))
        .unwrap();
    let Shape::Prism { points, z0, z1 } = &opening.shape else {
        panic!("the opening is a prism");
    };
    assert!(near_eq(*z0, 1.6) && near_eq(*z1, 5.4), "{z0} {z1}");
    let xs: Vec<f64> = points.iter().map(|p| p[0]).collect();
    let ys: Vec<f64> = points.iter().map(|p| p[1]).collect();
    assert!(near_eq(xs.iter().cloned().fold(f64::MAX, f64::min), 20.1));
    assert!(near_eq(xs.iter().cloned().fold(f64::MIN, f64::max), 22.9));
    assert!(near_eq(ys.iter().cloned().fold(f64::MAX, f64::min), -4.4));
    assert!(near_eq(ys.iter().cloned().fold(f64::MIN, f64::max), 4.4));
    assert!(opening.collide && !opening.show);
}

#[test]
fn the_near_probe_is_a_wide_cylinder_clipped_to_the_outline() {
    let all = built(&IDENTITY);
    let near = all
        .iter()
        .find(|p| {
            matches!(
                p.role,
                Role::HoleNear {
                    side: Side::Top,
                    ..
                }
            )
        })
        .unwrap();
    let Shape::Cylinder {
        center,
        r,
        z0,
        z1,
        clip,
    } = &near.shape
    else {
        panic!("the near probe is a cylinder");
    };
    assert_eq!(*center, [-15.0, 10.0]);
    assert!(near_eq(*r, 1.6 + 4.0 - 0.1));
    assert!(near_eq(*z0, 0.8) && near_eq(*z1, 2.6), "{z0} {z1}");
    assert_eq!(clip.as_ref().map(Vec::len), Some(4));
    assert!(!near.collide);
}

#[test]
fn every_hole_face_gets_four_ring_quadrants() {
    let rings: Vec<u8> = built(&IDENTITY)
        .iter()
        .filter_map(|p| match p.shape {
            Shape::Ring { quadrant, .. } => Some(quadrant),
            _ => None,
        })
        .collect();
    assert_eq!(rings, [0, 1, 2, 3, 0, 1, 2, 3]);
}

#[test]
fn a_connector_without_a_model_height_is_assumed_two_millimetres_tall() {
    let bare = BOARD.replace(r#""size":{"x":6,"y":8,"z":4.0}"#, r#""size":{}"#);
    let kit = build_kit(&bare, None).unwrap();
    let all = probes("main", &kit, &IDENTITY);
    let opening = all
        .iter()
        .find(|p| matches!(p.role, Role::Opening { .. }))
        .unwrap();
    let Shape::Prism { z0, z1, .. } = &opening.shape else {
        panic!("the opening is a prism");
    };
    assert!(near_eq(*z0, 0.8) && near_eq(*z1, 2.6), "{z0} {z1}");
}
