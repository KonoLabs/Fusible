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

#[test]
fn init_currency_writes_bom_yml() {
    let fixture = TestDir::new("init_currency");
    let out = fusible()
        .args(["init", "--currency", "usd", "--no-install"])
        .arg(&fixture.path)
        .output()
        .expect("binary should run");
    assert_eq!(out.status.code(), Some(0));
    let bom = std::fs::read_to_string(fixture.path.join("bom.yml")).unwrap();
    assert!(bom.contains("currency: USD"), "{bom}");
}

#[test]
fn init_rejects_an_unknown_currency_before_writing_anything() {
    let fixture = TestDir::new("init_bad_currency");
    let out = fusible()
        .args(["init", "--currency", "XYZ", "--no-install"])
        .arg(&fixture.path)
        .output()
        .expect("binary should run");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("unknown currency"));
    assert!(!fixture.path.join("fusible.yml").exists());
}

#[test]
fn doctor_json_lists_bun_node_tsci_and_exits_zero_with_nothing_installed() {
    let out = fusible()
        .args(["doctor", "--json"])
        .env("PATH", "")
        .env_remove("FUSIBLE_BUN")
        .output()
        .expect("binary should run");
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    for tool in ["bun", "node", "tsci"] {
        assert!(json[tool].is_null(), "{tool}: {json}");
    }
}

#[cfg(unix)]
#[test]
fn doctor_json_reports_the_bun_named_by_fusible_bun() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = TestDir::new("doctor_json_bun");
    let bun = fixture.path.join("bun");
    std::fs::write(&bun, "#!/bin/sh\necho 1.2.3\n").unwrap();
    std::fs::set_permissions(&bun, std::fs::Permissions::from_mode(0o755)).unwrap();
    let out = fusible()
        .args(["doctor", "--json"])
        .env("PATH", "")
        .env("FUSIBLE_BUN", &bun)
        .output()
        .expect("binary should run");
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["bun"]["path"], bun.to_str().unwrap());
    assert_eq!(json["bun"]["version"], "1.2.3");
}
