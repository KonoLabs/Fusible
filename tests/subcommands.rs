//! Integration tests for `check` and manifest discovery.

mod common;

use common::{fusible, TestDir};

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
fn subcommand_check_blows_fuse_on_silicon_pin_mux_conflict() {
    let out = fusible()
        .args([
            "check",
            "examples/circuit-pinmux-violation.json",
            "--mcu",
            "stm32g431",
        ])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("the fuse blew"));
    assert!(stderr.contains("PA5"));
    assert!(stderr.contains("UART1_TX"));
}

#[test]
fn bare_invocation_prints_help_even_with_a_manifest() {
    let fixture = TestDir::new("manifest_check");
    std::fs::write(
        fixture.path.join("board.json"),
        include_str!("../examples/circuit.json"),
    )
    .unwrap();
    std::fs::write(
        fixture.path.join("fusible.yml"),
        "name: auto-check\nhardware:\n  entry: board.json\n",
    )
    .unwrap();

    let bare = fusible()
        .current_dir(&fixture.path)
        .output()
        .expect("binary should run");
    assert_eq!(bare.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bare.stderr).contains("Usage: fusible"));

    let check = fusible()
        .current_dir(&fixture.path)
        .arg("check")
        .output()
        .expect("binary should run");
    assert_eq!(check.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&check.stdout).contains("fuse intact"));
}

#[test]
fn malformed_manifest_is_a_tool_error() {
    let fixture = TestDir::new("bad_manifest");
    std::fs::write(fixture.path.join("fusible.yml"), "hardware: [unclosed\n").unwrap();
    let out = fusible()
        .current_dir(&fixture.path)
        .args(["check", "board.json"])
        .output()
        .expect("binary should run");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn manifest_in_a_parent_directory_resolves_its_own_paths() {
    let fixture = TestDir::new("parent_manifest");
    std::fs::write(
        fixture.path.join("board.json"),
        include_str!("../examples/circuit.json"),
    )
    .unwrap();
    std::fs::write(
        fixture.path.join("fusible.yml"),
        "hardware:\n  entry: board.json\n",
    )
    .unwrap();
    let sub = fixture.path.join("firmware");
    std::fs::create_dir_all(&sub).unwrap();
    let out = fusible()
        .current_dir(&sub)
        .arg("check")
        .output()
        .expect("binary should run");
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn stale_circuit_json_is_refused_without_a_runner() {
    let fixture = TestDir::new("stale_json");
    let dist = fixture.path.join("dist");
    std::fs::create_dir_all(&dist).unwrap();
    let json = dist.join("circuit.json");
    let tsx = fixture.path.join("board.tsx");
    let run = || {
        fusible()
            .current_dir(&fixture.path)
            .env("PATH", "")
            .args(["check", "board.tsx"])
            .output()
            .expect("binary should run")
            .status
            .code()
    };

    std::fs::write(&json, include_str!("../examples/circuit.json")).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(&tsx, "export default () => <board />\n").unwrap();
    assert_eq!(run(), Some(2));

    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(&json, include_str!("../examples/circuit.json")).unwrap();
    assert_eq!(run(), Some(0));
}

const MINIMAL_KICAD_PCB: &str =
    r#"(kicad_pcb (footprint "R" (layer "F.Cu") (at 1 2) (property "Reference" "R1")))"#;

#[test]
fn subcommands_diff_and_sync_exit_2_on_truncated_circuit_json() {
    let dir = TestDir::new("kicad_invalid_json");
    let pcb = dir.path.join("board.kicad_pcb");
    let json = dir.path.join("circuit.json");
    std::fs::write(&pcb, MINIMAL_KICAD_PCB).unwrap();
    std::fs::write(&json, r#"[{"type": "source_component", "name": "R1""#).unwrap();

    for subcommand in ["diff", "sync"] {
        let out = fusible()
            .arg(subcommand)
            .arg(&pcb)
            .arg(&json)
            .output()
            .expect("binary should run");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(2), "{subcommand}: {stderr}");
        assert!(stderr.contains("invalid_circuit_json"), "{subcommand}");
        assert!(stderr.contains("circuit.json"), "{subcommand}");
    }
}
