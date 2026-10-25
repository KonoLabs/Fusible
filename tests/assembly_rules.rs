//! Integration tests for the cross-world rules of `fusible assembly`.
//!
//! They need bun and occt-wasm (installed on first use), so each test skips when bun is missing.
//! `FUSIBLE_REQUIRE_BUN=1` turns that skip into a failure.

mod common;
#[path = "assembly_rules/solids.rs"]
mod solids;
#[path = "assembly_rules/support.rs"]
mod support;

use common::fusible;
use support::{bun_available, project, report, run};

macro_rules! need_bun {
    () => {
        if !bun_available() {
            eprintln!("skipped: bun is not installed");
            return;
        }
    };
}

#[test]
fn a_body_through_the_board_blows_the_fuse_and_writes_no_model() {
    need_bun!();
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
    need_bun!();
    let dir = project("assembly_rules_clear", "[50, 0, 0]");
    let (code, stderr) = run(&dir.path, &["assembly", "-f", "all"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(dir.path.join("dist/assembly.glb").is_file());
    assert_eq!(report(&dir.path)["fuse"], "intact");
}

#[test]
fn check_runs_the_rules_when_every_board_is_checked() {
    need_bun!();
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
    let stderr = String::from_utf8_lossy(&out.stderr).to_lowercase();
    assert!(stderr.contains("bun"), "{stderr}");
}

#[test]
fn a_blown_fuse_removes_the_exports_and_report_of_the_previous_run() {
    need_bun!();
    let dir = project("assembly_rules_stale_blown", "[50, 0, 0]");
    let (code, stderr) = run(&dir.path, &["assembly", "-f", "all"]);
    assert_eq!(code, Some(0), "{stderr}");
    let manifest = std::fs::read_to_string(dir.path.join("fusible.yml")).unwrap();
    std::fs::write(
        dir.path.join("fusible.yml"),
        manifest.replace("[50, 0, 0]", "[0, 0, 0]"),
    )
    .unwrap();
    let (code, stderr) = run(&dir.path, &["assembly", "-f", "all"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(!dir.path.join("dist/assembly.glb").exists());
    assert!(!dir.path.join("dist/assembly.step").exists());
    assert_eq!(report(&dir.path)["fuse"], "blown");
}

#[test]
fn a_tool_error_leaves_no_report_or_export_behind() {
    let dir = project("assembly_rules_stale_error", "[50, 0, 0]");
    let parts = dir.path.join("dist/assembly");
    std::fs::create_dir_all(&parts).unwrap();
    for stale in [
        "dist/assembly/check.json",
        "dist/assembly/preview.glb",
        "dist/assembly.glb",
        "dist/assembly.step",
    ] {
        std::fs::write(dir.path.join(stale), "{\"fuse\":\"intact\"}").unwrap();
    }
    let out = fusible()
        .current_dir(&dir.path)
        .env("PATH", "")
        .env_remove("FUSIBLE_BUN")
        .args(["assembly", "-f", "all"])
        .output()
        .expect("binary should run");
    assert_eq!(out.status.code(), Some(2));
    assert!(!parts.join("check.json").exists());
    assert!(!parts.join("preview.glb").exists());
    assert!(!dir.path.join("dist/assembly.glb").exists());
    assert!(!dir.path.join("dist/assembly.step").exists());
}
