//! Exit-code contract tests: the promise the whole product is built on.
//!
//! A violation must exit 1 (fuse blown), a clean circuit 0, and a tool
//! failure 2 — otherwise a phase-2 `fusible && generate-pins` chain cannot
//! tell "the circuit is bad" from "fusible could not read its input".

mod common;

use common::{fusible, TestDir};

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

fn oversized_schematic_component() -> serde_json::Value {
    serde_json::json!({
        "type": "schematic_component",
        "schematic_component_id": "schematic_component_0",
        "source_component_id": "source_component_0",
        "refdes": "U1",
        "center": { "x": 700.0, "y": -1000.0 }
    })
}

#[test]
fn schematic_scale_alone_warns_and_exits_zero() {
    let dir = TestDir::new("scale_warning");
    let path = dir.path.join("circuit.json");
    let circuit = serde_json::json!([oversized_schematic_component()]);
    std::fs::write(&path, circuit.to_string()).expect("write circuit");

    let out = fusible()
        .arg("--input")
        .arg(&path)
        .output()
        .expect("the binary must run");

    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("E_SCHEMATIC_SCALE_EXCESSIVE"),
        "got: {stderr}"
    );
    assert!(!stderr.contains("the fuse blew"), "got: {stderr}");
}

#[test]
fn schematic_scale_next_to_a_real_violation_still_blows_the_fuse() {
    let dir = TestDir::new("scale_with_violation");
    let path = dir.path.join("circuit.json");
    let floating = std::fs::read_to_string("examples/circuit-flottant.json").expect("fixture");
    let mut elements: Vec<serde_json::Value> = serde_json::from_str(&floating).expect("json");
    elements.push(oversized_schematic_component());
    std::fs::write(&path, serde_json::Value::Array(elements).to_string()).expect("write circuit");

    let out = fusible()
        .arg("--input")
        .arg(&path)
        .output()
        .expect("the binary must run");

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unconnected_net"), "got: {stderr}");
    assert!(
        stderr.contains("E_SCHEMATIC_SCALE_EXCESSIVE"),
        "got: {stderr}"
    );
}
