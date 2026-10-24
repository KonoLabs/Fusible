//! Integration tests for `doctor --json`.

mod common;

use common::fusible;
#[cfg(unix)]
use common::TestDir;

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
