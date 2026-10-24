//! Integration tests for `links:` and the `harness_mismatch` rule.

mod common;

use common::{fusible, TestDir};

fn board(refdes: &str, net: &str) -> String {
    format!(
        r#"[
  {{ "type": "source_component", "component_id": "{refdes}", "kind": "chip" }},
  {{ "type": "source_component", "component_id": "R1", "kind": "resistor" }},
  {{ "type": "source_net", "net_id": "n1", "name": "{net}" }},
  {{ "type": "source_port", "port_id": "{refdes}.1", "name": "1", "net_id": "n1" }},
  {{ "type": "source_port", "port_id": "R1.1", "name": "1", "net_id": "n1" }}
]"#
    )
}

fn project(sensor_net: &str) -> TestDir {
    let dir = TestDir::new("links");
    std::fs::write(dir.path.join("main.json"), board("J2", "GND")).unwrap();
    std::fs::write(dir.path.join("sensor.json"), board("J1", sensor_net)).unwrap();
    std::fs::write(
        dir.path.join("fusible.yml"),
        "boards:\n  main: {circuit_json: main.json}\n  sensor: {circuit_json: sensor.json}\n\
         links:\n  cable:\n    from: main.J2\n    to: sensor.J1\n    wires: [{a: 1, b: 1}]\n",
    )
    .unwrap();
    dir
}

#[test]
fn matching_link_keeps_the_fuse_intact() {
    let dir = project("GND");
    let out = fusible()
        .arg("check")
        .current_dir(&dir.path)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn mismatched_link_blows_the_fuse() {
    let dir = project("3V3");
    let out = fusible()
        .arg("check")
        .current_dir(&dir.path)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("harness_mismatch") && stderr.contains("[link cable]"),
        "{stderr}"
    );
}
