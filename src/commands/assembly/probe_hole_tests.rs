use super::place::IDENTITY;
use super::probe::{Role, Shape};
use super::probe_tests::{built, near_eq, BOARD};
use super::probe_warnings::warnings;
use crate::commands::body::build_kit;

#[test]
fn the_bore_probe_runs_through_the_board_and_the_boss_range_inside_the_hole() {
    let all = built(&IDENTITY);
    let bore = all
        .iter()
        .find(|p| matches!(p.role, Role::HoleBore { .. }))
        .unwrap();
    let Shape::Cylinder {
        r, z0, z1, clip, ..
    } = &bore.shape
    else {
        panic!("the bore is a cylinder");
    };
    assert!(near_eq(*r, 1.1), "{r}");
    assert!(near_eq(*z0, -2.6) && near_eq(*z1, 2.6), "{z0} {z1}");
    assert!(clip.is_none() && bore.collide);
}

#[test]
fn a_connector_without_height_or_keep_out_is_warned_about() {
    let bare = BOARD.replace(r#""size":{"x":6,"y":8,"z":4.0}"#, r#""size":{}"#);
    let kit = build_kit(&bare, None).unwrap();
    let lines = warnings("main", &kit);
    assert!(
        lines.iter().any(|l| l.contains("no 3D height")),
        "{lines:?}"
    );
    assert!(lines.iter().any(|l| l.contains("no keep-out")), "{lines:?}");
    let kit = build_kit(BOARD, None).unwrap();
    assert!(warnings("main", &kit).is_empty());
}
