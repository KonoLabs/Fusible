use super::rules::{violations, Measurement, HOLE_MISALIGNED};
use super::rules_support::{board_probes, hit, response, rules};

fn quadrants(count: usize) -> Vec<Measurement> {
    (0..count)
        .map(|q| hit(&format!("main:hole:0:top:q{q}"), 1.0))
        .collect()
}

#[test]
fn four_quadrant_hits_mean_the_boss_is_aligned() {
    let mut hits = vec![hit("main:hole:0:top:near", 5.0)];
    hits.extend(quadrants(4));
    assert!(violations(&board_probes(), &response(hits)).is_empty());
}

#[test]
fn a_near_hit_with_two_quadrant_misses_is_a_misaligned_hole() {
    let mut hits = vec![hit("main:hole:0:top:near", 5.0)];
    hits.extend(quadrants(2));
    let found = violations(&board_probes(), &response(hits));
    assert_eq!(rules(&found), [HOLE_MISALIGNED]);
    assert!(
        found[0].message.contains("(-15.00, 10.00) on the top side"),
        "{}",
        found[0].message
    );
}

#[test]
fn a_near_miss_says_nothing_even_when_every_quadrant_misses() {
    assert!(violations(&board_probes(), &response(vec![])).is_empty());
}

#[test]
fn the_other_face_of_the_hole_is_judged_on_its_own() {
    let mut hits = quadrants(4);
    hits.push(hit("main:hole:0:bottom:near", 5.0));
    let found = violations(&board_probes(), &response(hits));
    assert_eq!(rules(&found), [HOLE_MISALIGNED]);
    assert!(found[0].message.contains("bottom side"));
}
