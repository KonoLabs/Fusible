//! Integration tests for subcommands (`check`, `init`, etc.).

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

fn fusible() -> Command {
    Command::new(env!("CARGO_BIN_EXE_fusible"))
}

struct TestDir {
    pub path: std::path::PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let path = std::env::temp_dir().join(format!("fusible_integration_{pid}_{id}_{name}"));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("failed to create test dir");
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[test]
fn subcommand_check_validates_intact_circuit() {
    let out = fusible()
        .args(["check", "examples/circuit.json", "--mcu", "stm32g431"])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("fuse intact"));
    assert!(stdout.contains("stm32g431"));
}

#[test]
fn subcommand_check_detects_blown_fuse() {
    let out = fusible()
        .args(["check", "examples/circuit-floating.json"])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("the fuse blew"));
}

#[test]
fn subcommand_init_creates_valid_manifest() {
    let fixture = TestDir::new("subcmd_init");

    let out = fusible()
        .args(["init", "--name", "test-project"])
        .current_dir(&fixture.path)
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(0));
    assert!(fixture.path.join("fusible.yml").exists());

    let content = std::fs::read_to_string(fixture.path.join("fusible.yml")).unwrap();
    assert!(content.contains("name: test-project"));
}

#[test]
fn runs_check_from_manifest_with_no_arguments() {
    let fixture = TestDir::new("manifest_check");
    let circuit_json = include_str!("../examples/circuit.json");
    std::fs::write(fixture.path.join("board.json"), circuit_json).unwrap();

    let manifest = r#"
name: auto-check
hardware:
  entry: board.json
"#;
    std::fs::write(fixture.path.join("fusible.yml"), manifest).unwrap();

    let out = fusible()
        .current_dir(&fixture.path)
        .output()
        .expect("binary should run");

    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "stdout: {stdout}, stderr: {stderr}");
    assert!(stdout.contains("fuse intact"));
}
