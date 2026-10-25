use super::rules::{violations, Measurement, HOLE_MISALIGNED};
use super::rules_support::{board_probes, hit, response, rules};

fn quadrants(count: usize) -> Vec<Measurement> {
    (0..count)
        .map(|q| hit(&format!("main:hole:0:top:q{q}"), 1.0))
        .collect()
}

fn boss(count: usize) -> Vec<Measurement> {
    let mut hits = vec![hit("main:hole:0:top:near", 5.0)];
    hits.extend(quadrants(count));
    hits
}

#[test]
fn an_aligned_boss_with_a_clear_bore_is_not_a_violation() {
    assert!(violations(&board_probes(), &response(boss(4))).is_empty());
}

#[test]
fn a_counterbore_that_misses_two_quadrants_is_not_a_violation() {
    assert!(violations(&board_probes(), &response(boss(2))).is_empty());
}

#[test]
fn a_near_hit_with_one_quadrant_is_a_misaligned_hole() {
    let found = violations(&board_probes(), &response(boss(1)));
    assert_eq!(rules(&found), [HOLE_MISALIGNED]);
    assert!(
        found[0].message.contains("(-15.00, 10.00) on the top side"),
        "{}",
        found[0].message
    );
}

#[test]
fn material_in_the_bore_is_a_misaligned_hole_even_with_a_full_ring() {
    let mut hits = boss(4);
    hits.push(hit("main:hole:0:bore", 2.0));
    let found = violations(&board_probes(), &response(hits));
    assert_eq!(rules(&found), [HOLE_MISALIGNED]);
    assert!(
        found[0]
            .message
            .contains("inside the hole at (-15.00, 10.00)"),
        "{}",
        found[0].message
    );
}

#[test]
fn a_boss_offset_by_a_millimetre_and_a_half_fills_the_bore() {
    let mut hits = boss(2);
    hits.push(hit("main:hole:0:bore", 0.8));
    let found = violations(&board_probes(), &response(hits));
    assert_eq!(rules(&found), [HOLE_MISALIGNED]);
}

#[test]
fn the_other_face_of_the_hole_is_judged_on_its_own() {
    let mut hits = quadrants(4);
    hits.push(hit("main:hole:0:bottom:near", 5.0));
    let found = violations(&board_probes(), &response(hits));
    assert_eq!(rules(&found), [HOLE_MISALIGNED]);
    assert!(found[0].message.contains("bottom side"));
}
