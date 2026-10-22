//! Circuit JSON builders for end-to-end routing tests.

/// A source port, its PCB port, and a 1 mm square SMT pad at `(x, y)`.
pub fn pad(id: &str, x: f64, y: f64, layer: &str) -> String {
    format!(
        r#"{{ "type": "source_port", "source_port_id": "sp_{id}", "name": "{id}" }},
        {{ "type": "pcb_port", "pcb_port_id": "pp_{id}", "source_port_id": "sp_{id}",
           "x": {x}, "y": {y}, "layers": ["{layer}"] }},
        {{ "type": "pcb_smtpad", "pcb_smtpad_id": "pad_{id}", "pcb_port_id": "pp_{id}",
           "shape": "rect", "x": {x}, "y": {y}, "width": 1.0, "height": 1.0,
           "layer": "{layer}" }}"#
    )
}

/// A source trace joining pads `a` and `b` into one net.
pub fn connect(id: &str, a: &str, b: &str) -> String {
    format!(
        r#"{{ "type": "source_trace", "source_trace_id": "st_{id}",
           "connected_source_port_ids": ["sp_{a}", "sp_{b}"], "connected_source_net_ids": [] }}"#
    )
}

/// A centered board of `board` width and height holding `parts`.
pub fn circuit(board: (f64, f64), parts: &[String]) -> String {
    format!(
        r#"[{{ "type": "pcb_board", "width": {}, "height": {},
             "center": {{ "x": 0, "y": 0 }} }}, {}]"#,
        board.0,
        board.1,
        parts.join(",")
    )
}

/// Two nets whose straight routes would cross at the origin.
pub fn crossing_nets() -> String {
    circuit(
        (30.0, 30.0),
        &[
            pad("a1", -5.0, 0.0, "top"),
            pad("a2", 5.0, 0.0, "top"),
            pad("b1", 0.0, -5.0, "top"),
            pad("b2", 0.0, 5.0, "top"),
            connect("a", "a1", "a2"),
            connect("b", "b1", "b2"),
        ],
    )
}
