//! Exit-code contract tests: the promise the whole product is built on.
//!
//! A violation must exit 1 (fuse blown), a clean circuit 0, and a tool
//! failure 2 — otherwise a phase-2 `fusible && generate-pins` chain cannot
//! tell "the circuit is bad" from "fusible could not read its input".

use std::process::Command;

fn fusible() -> Command {
    Command::new(env!("CARGO_BIN_EXE_fusible"))
}

#[test]
fn intact_circuit_exits_zero_and_says_so_on_stdout() {
    let out = fusible()
        .args(["--input", "examples/circuit.json", "--mcu", "stm32g431"])
        .output()
        .expect("the binary must run");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("fuse intact"), "got: {stdout}");
    assert!(stdout.contains("stm32g431"), "got: {stdout}");
}

#[test]
fn violations_exit_one_with_the_fuse_blown_report() {
    let out = fusible()
        .args(["--input", "examples/circuit-flottant.json"])
        .output()
        .expect("the binary must run");

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("the fuse blew"), "got: {stderr}");
    assert!(stderr.contains("unconnected_net"), "got: {stderr}");
}

#[test]
fn malformed_json_exits_two_as_a_tool_error() {
    let out = fusible()
        .args(["--input", "examples/circuit-broken.json"])
        .output()
        .expect("the binary must run");

    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("invalid Circuit JSON"), "got: {stderr}");
}

#[test]
fn missing_file_exits_two() {
    let out = fusible()
        .args(["--input", "does-not-exist.json"])
        .output()
        .expect("the binary must run");

    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("cannot read the Circuit JSON"),
        "got: {stderr}"
    );
}
