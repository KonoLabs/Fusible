use std::path::Path;

use super::hub::ClientHub;
use super::pipeline::execute_pipeline;

#[test]
fn pipeline_runs_on_valid_circuit_json() {
    let payload = execute_pipeline(Path::new("examples/circuit.json"));
    assert!(!payload.fuse_blown);
    assert!(payload.diagnostics.is_empty());
    assert!(!payload.pcb_svg.is_empty());
    assert!(!payload.schematic_svg.is_empty());
    assert!(payload.circuit_json.is_some());
    assert!(payload.pcb_svg.contains("<svg"));
    assert!(payload.schematic_svg.contains("<svg"));
}

#[test]
fn pipeline_flags_blown_fuse_on_floating_net() {
    let payload = execute_pipeline(Path::new("examples/circuit-floating.json"));
    assert!(payload.fuse_blown);
    assert!(!payload.diagnostics.is_empty());
    assert!(payload
        .diagnostics
        .iter()
        .any(|d| d.rule == "unconnected_net"));
}

#[test]
fn pipeline_handles_missing_file_gracefully() {
    let payload = execute_pipeline(Path::new("nonexistent.json"));
    assert!(payload.fuse_blown);
    assert_eq!(payload.diagnostics[0].rule, "source_error");
}

#[test]
fn client_hub_broadcast_and_pruning() {
    let hub = ClientHub::new();
    let (tx1, rx1) = std::sync::mpsc::channel();
    let (tx2, rx2) = std::sync::mpsc::channel();

    hub.add_client(tx1);
    hub.add_client(tx2);

    hub.broadcast(r#"{"test": true}"#);

    let msg1 = String::from_utf8(rx1.recv().unwrap()).unwrap();
    assert!(msg1.starts_with("data: {\"test\": true}\n\n"));

    let msg2 = String::from_utf8(rx2.recv().unwrap()).unwrap();
    assert!(msg2.starts_with("data: {\"test\": true}\n\n"));

    drop(rx2);
    hub.broadcast(r#"{"test": 2}"#);

    let msg3 = String::from_utf8(rx1.recv().unwrap()).unwrap();
    assert!(msg3.starts_with("data: {\"test\": 2}\n\n"));
}
