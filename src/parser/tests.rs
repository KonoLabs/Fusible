//! Tests for the Circuit JSON data model: heterogeneous array parsing, the
//! snake_case/camelCase alias policy, and the span mapping that turns a
//! serde_json (line, column) into a miette byte offset.
//!
//! Sibling file of `parser.rs` (declared `#[cfg(test)] mod tests`): keeping
//! tests out of the module file is what holds `parser.rs` under filet's
//! functions-per-file budget.

use std::path::Path;

use super::diagnostic::byte_offset_at;
use super::*;

fn parse_ok(json: &str) -> Circuit {
    parse(json, Path::new("circuit.json")).expect("fixture must parse")
}

#[test]
fn parses_a_heterogeneous_array() {
    let json = r#"[
        {"type": "source_component", "component_id": "U1", "kind": "chip"},
        {"type": "source_net", "net_id": "n1", "name": "GND"},
        {"type": "source_port", "port_id": "U1.1", "name": "VSS", "net_id": "n1"},
        {"type": "source_pad", "pad_id": "p1", "net_id": "n1"},
        {"type": "source_trace", "trace_id": "t1", "connected_source_port_ids": ["U1.1", "U1.2"]},
        {"type": "silkscreen_text", "text": "not modeled in phase 1"}
    ]"#;

    let circuit = parse_ok(json);

    assert_eq!(circuit.elements.len(), 6);
    assert_eq!(circuit.components().count(), 1);
    assert_eq!(circuit.nets().count(), 1);
    assert_eq!(circuit.ports().count(), 1);
    assert_eq!(circuit.pads().count(), 1);
    assert_eq!(circuit.traces().count(), 1);
    // The unknown type is ignored, not rejected.
    assert!(matches!(circuit.elements[5], CircuitElement::Unknown));
}

#[test]
fn accepts_canonical_snake_case_and_defensive_camelcase_aliases() {
    // Canonical circuit-json spelling — the one real tscircuit emits.
    let json = r#"[{"type": "source_trace", "source_trace_id": "t1",
             "connected_source_port_ids": ["U1.1"] }]"#;
    let circuit = parse_ok(json);
    let trace = circuit.traces().next().expect("a trace");
    assert_eq!(trace.trace_id.as_deref(), Some("t1"));
    assert_eq!(trace.connected_source_port_ids, vec!["U1.1"]);

    // Defensive camelCase tolerance for alternative generators.
    let json = r#"[{"type": "source_component", "sourceComponentId": "R1", "ftype": "resistor"}]"#;
    let circuit = parse_ok(json);
    let component = circuit.components().next().expect("a component");
    assert_eq!(component.component_id.as_deref(), Some("R1"));
    assert_eq!(component.kind.as_deref(), Some("resistor"));
}

#[test]
fn recognized_elements_without_identifiers_parse_to_none() {
    // Deserialization stays tolerant; the linter's `malformed_element` rule
    // is what refuses to validate this input silently.
    let json = r#"[
        {"type": "source_net", "name": "GND"},
        {"type": "source_port", "name": "SDA"}
    ]"#;

    let circuit = parse_ok(json);
    let net = circuit.nets().next().expect("a net");
    let port = circuit.ports().next().expect("a port");
    assert!(net.net_id.is_none());
    assert!(port.port_id.is_none());
    assert!(port.net_id.is_none());
}

#[test]
fn element_errors_name_the_element_and_point_at_its_line() {
    let source = "[\n{\"type\": \"source_net\", \"source_net_id\": \"n1\"},\n{\"type\": \"source_port\", \"pin_number\": \"x\"}\n]";
    let line3 = source.find("{\"type\": \"source_port\"").expect("line 3");

    let err = parse(source, Path::new("circuit.json")).expect_err("bad pin_number");

    assert!(
        err.message.starts_with("element #1:"),
        "got: {}",
        err.message
    );
    assert!(
        err.span.offset() >= line3,
        "span {} before line 3",
        err.span.offset()
    );
}

#[test]
fn object_root_and_empty_input_are_rejected() {
    let err = parse("{\"type\": \"source_net\"}", Path::new("circuit.json"))
        .expect_err("an object root must not parse");
    assert!(err.message.contains("sequence"), "got: {}", err.message);

    let err = parse("", Path::new("circuit.json")).expect_err("empty input must not parse");
    assert!(err.message.contains("EOF"), "got: {}", err.message);
}

#[test]
fn span_mapping_handles_multiline_crlf_and_multibyte() {
    // LF, error on line 3: `{` at 11 ... `}` at 17 (column 7).
    let lf = "[\n{\"a\": 1,\n{\"b\": }\n]";
    let offset = byte_offset_at(lf, 3, 7);
    assert_eq!(offset, 17);
    assert_eq!(lf.as_bytes()[offset], b'}');

    // CRLF: the `\r` bytes must stay in the running offset, or the label
    // drifts left by one byte per preceding line. Line 3 starts at 13,
    // `}` sits at 19 (column 7).
    let crlf = "[\r\n{\"a\": 1,\r\n{\"b\": }\r\n]";
    let offset = byte_offset_at(crlf, 3, 7);
    assert_eq!(offset, 19);
    assert_eq!(crlf.as_bytes()[offset], b'}');

    // Multi-byte: serde columns are byte counts. Line 2 of this source is
    // `{"a": "éé", "b": }` — `}` at byte column 20, and column 9 lands in
    // the middle of the first `é` and must snap back to its boundary.
    let multi = "[\n{\"a\": \"éé\", \"b\": }\n]";
    let offset = byte_offset_at(multi, 2, 20);
    assert_eq!(offset, 21);
    assert_eq!(multi.as_bytes()[offset], b'}');

    let offset = byte_offset_at(multi, 2, 9);
    assert!(
        multi.is_char_boundary(offset),
        "offset {offset} splits a char"
    );

    // A line beyond the file lands at EOF instead of panicking.
    assert_eq!(byte_offset_at(lf, 99, 1), lf.len());
}
