use super::place::{apply, compose, placement, translation, IDENTITY};
use crate::config::PlacementConfig;

fn close(a: [f64; 3], b: [f64; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-9)
}

fn at(at: [f64; 3], rotate: [f64; 3]) -> PlacementConfig {
    PlacementConfig {
        at: Some(at),
        rotate: Some(rotate),
    }
}

#[test]
fn an_empty_placement_is_the_identity() {
    assert_eq!(placement(&PlacementConfig::default()), IDENTITY);
}

#[test]
fn a_translation_moves_every_point() {
    let m = placement(&at([1.0, 2.0, 3.0], [0.0, 0.0, 0.0]));
    assert!(close(apply(&m, [10.0, 0.0, 0.0]), [11.0, 2.0, 3.0]));
}

#[test]
fn rotations_turn_around_x_then_y_then_z_before_the_translation() {
    let z90 = placement(&at([0.0, 0.0, 5.0], [0.0, 0.0, 90.0]));
    assert!(close(apply(&z90, [1.0, 0.0, 0.0]), [0.0, 1.0, 5.0]));
    let x_then_z = placement(&at([0.0, 0.0, 0.0], [90.0, 0.0, 90.0]));
    assert!(close(apply(&x_then_z, [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]));
    assert!(close(apply(&x_then_z, [1.0, 0.0, 0.0]), [0.0, 1.0, 0.0]));
}

#[test]
fn a_body_follows_its_board_after_its_own_offset() {
    let board = placement(&at([100.0, 0.0, 0.0], [0.0, 0.0, 90.0]));
    let body = compose(&board, &translation([0.0, 0.0, -4.2]));
    assert!(close(apply(&body, [1.0, 0.0, 4.2]), [100.0, 1.0, 0.0]));
    assert_eq!(compose(&IDENTITY, &board), board);
}
