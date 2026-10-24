//! Integration tests for `init` and `doctor`.

mod common;

use common::{fusible, TestDir};

#[test]
fn subcommand_init_creates_valid_manifest() {
    let fixture = TestDir::new("subcmd_init");

    let out = fusible()
        .args(["init", "--name", "test-project", "--no-install"])
        .current_dir(&fixture.path)
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(0));
    assert!(fixture.path.join("fusible.yml").exists());
    assert!(fixture.path.join("hardware/index.tsx").exists());
    assert!(fixture.path.join("hardware/package.json").exists());
    assert!(fixture.path.join("hardware/tsconfig.json").exists());

    let content = std::fs::read_to_string(fixture.path.join("fusible.yml")).unwrap();
    assert!(content.contains("name: test-project"));
}

#[test]
fn subcommand_init_ignores_the_fusible_work_folder() {
    let fixture = TestDir::new("subcmd_init_gitignore");
    let out = fusible()
        .args(["init", "--name", "ignored", "--no-install"])
        .current_dir(&fixture.path)
        .output()
        .expect("binary should run");
    assert_eq!(out.status.code(), Some(0));
    let ignore = std::fs::read_to_string(fixture.path.join(".gitignore")).unwrap();
    assert!(ignore.lines().any(|line| line == ".fusible/"));
}

#[test]
fn subcommand_init_with_path_and_mcu() {
    let fixture = TestDir::new("subcmd_init_path");
    let target = fixture.path.join("sensor_node");

    let out = fusible()
        .args([
            "init",
            target.to_str().unwrap(),
            "--name",
            "sensor-node",
            "--mcu",
            "stm32g431",
            "--no-install",
        ])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(0));
    assert!(target.join("fusible.yml").exists());
    assert!(target.join("hardware/index.tsx").exists());
    assert!(target.join("hardware/package.json").exists());

    let manifest = std::fs::read_to_string(target.join("fusible.yml")).unwrap();
    assert!(manifest.contains("name: sensor-node"));
    assert!(manifest.contains("mcu: stm32g431"));
}

#[test]
fn init_keeps_existing_hardware_files() {
    let fixture = TestDir::new("init_keep");
    let hw = fixture.path.join("hardware");
    std::fs::create_dir_all(&hw).unwrap();
    std::fs::write(hw.join("index.tsx"), "mine").unwrap();
    let out = fusible()
        .args(["init", "--no-install"])
        .arg(&fixture.path)
        .output()
        .expect("binary should run");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        std::fs::read_to_string(hw.join("index.tsx")).unwrap(),
        "mine"
    );
    assert!(hw.join("package.json").exists());
}

#[test]
fn subcommand_doctor_reports_health() {
    let out = fusible().arg("doctor").output().expect("binary should run");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Fusible Toolchain Doctor"));
    assert!(stdout.contains("Runtime prerequisites"));
}
