//! Unit tests for the native Gerber and Excellon generator.

mod layers;
mod shapes;
mod silkscreen;
mod validation;
mod writers;

/// Wraps extra Circuit JSON elements after a 50 x 40 mm board centred on the origin.
fn board_with(extra: &str) -> String {
    format!(
        r#"[{{ "type": "pcb_board", "pcb_board_id": "b0", "width": 50, "height": 40,
            "center": {{ "x": 0, "y": 0 }} }}{extra}]"#
    )
}
