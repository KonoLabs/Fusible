//! Integration tests for the hole rule of `fusible assembly`, on real STL geometry.
//!
//! They need bun and occt-wasm, so each test skips when bun is missing.
//! `FUSIBLE_REQUIRE_BUN=1` turns that skip into a failure.

mod common;
#[path = "assembly_rules/solids.rs"]
mod solids;
#[path = "assembly_rules/support.rs"]
mod support;

use solids::{box_triangles, reversed, tube_triangles};
use support::{bun_available, project_with, report, run, HOLE_BOARD};

macro_rules! need_bun {
    () => {
        if !bun_available() {
            eprintln!("skipped: bun is not installed");
            return;
        }
    };
}

fn boss_over_the_hole(name: &str, at: &str, bore: f64) -> common::TestDir {
    project_with(name, at, HOLE_BOARD, &tube_triangles(5.0, bore, -5.0, 5.0))
}

#[test]
fn a_boss_lined_up_with_a_clear_bore_keeps_the_fuse_intact() {
    need_bun!();
    let dir = boss_over_the_hole("assembly_rules_boss_aligned", "[0, 0, 5.8]", 1.55);
    let (code, stderr) = run(&dir.path, &["assembly"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(report(&dir.path)["fuse"], "intact");
}

#[test]
fn a_boss_offset_by_a_millimetre_and_a_half_blows_the_fuse() {
    need_bun!();
    let dir = boss_over_the_hole("assembly_rules_boss_offset", "[1.5, 0, 5.8]", 1.55);
    let (code, stderr) = run(&dir.path, &["assembly"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("hole_misaligned"), "{stderr}");
}

#[test]
fn a_bore_narrower_than_the_hole_blows_the_fuse() {
    need_bun!();
    let dir = boss_over_the_hole("assembly_rules_boss_narrow", "[0, 0, 5.8]", 0.8);
    let (code, stderr) = run(&dir.path, &["assembly"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("hole_misaligned"), "{stderr}");
}

#[test]
fn an_inside_out_stl_is_not_measured_as_a_solid() {
    need_bun!();
    let dir = project_with(
        "assembly_rules_inside_out",
        "[50, 0, 0]",
        support::BOARD,
        &reversed(box_triangles(5.0, -5.0, 5.0)),
    );
    let (code, stderr) = run(&dir.path, &["assembly"]);
    assert!(
        code == Some(2) && stderr.contains("not a solid"),
        "{code:?} {stderr}"
    );
}
