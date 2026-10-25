//! Integration tests for the cross-world rules of `fusible assembly`.
//!
//! They need bun and occt-wasm (installed on first use), so each test skips when bun is missing.

mod common;
#[path = "assembly_rules/support.rs"]
mod support;

use common::fusible;
use support::{bun_available, project, report, run};

#[test]
fn a_body_through_the_board_blows_the_fuse_and_writes_no_model() {
    if !bun_available() {
        eprintln!("skipped: bun is not installed");
        return;
    }
    let dir = project("assembly_rules_overlap", "[0, 0, 0]");
    let (code, stderr) = run(&dir.path, &["assembly", "-f", "all"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("body_interference"), "{stderr}");
    assert!(!dir.path.join("dist/assembly.glb").exists());
    assert!(!dir.path.join("dist/assembly.step").exists());
    assert!(dir.path.join("dist/assembly/preview.glb").is_file());
    let report = report(&dir.path);
    assert_eq!(report["fuse"], "blown");
    assert_eq!(report["violations"][0]["rule"], "body_interference");
    assert_eq!(
        report["violations"][0]["parts"],
        serde_json::json!(["box", "main"])
    );
}

#[test]
fn the_same_body_moved_away_keeps_the_fuse_intact() {
    if !bun_available() {
        eprintln!("skipped: bun is not installed");
        return;
    }
    let dir = project("assembly_rules_clear", "[50, 0, 0]");
    let (code, stderr) = run(&dir.path, &["assembly", "-f", "all"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(dir.path.join("dist/assembly.glb").is_file());
    assert_eq!(report(&dir.path)["fuse"], "intact");
}

#[test]
fn check_runs_the_rules_when_every_board_is_checked() {
    if !bun_available() {
        eprintln!("skipped: bun is not installed");
        return;
    }
    let dir = project("assembly_rules_check", "[0, 0, 0]");
    let (code, stderr) = run(&dir.path, &["check"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("body_interference"), "{stderr}");
    assert!(!dir.path.join("dist").exists());
}

#[test]
fn a_declared_assembly_without_bun_is_a_tool_error_not_a_blown_fuse() {
    let dir = project("assembly_rules_nobun", "[0, 0, 0]");
    let out = fusible()
        .current_dir(&dir.path)
        .env("PATH", "")
        .env_remove("FUSIBLE_BUN")
        .args(["check"])
        .output()
        .expect("binary should run");
    assert_eq!(out.status.code(), Some(2));
}
