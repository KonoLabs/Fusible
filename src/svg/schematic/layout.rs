//! Grid placement of component cards and their pin anchors.

use std::collections::HashMap;

use super::{component_id, component_ports};
use crate::parser::{Circuit, Component};

pub(super) const COMP_W: f64 = 40.0;
pub(super) const PIN_H: f64 = 6.0;
const GAP_X: f64 = 35.0;
const ORIGIN: f64 = 20.0;

/// Component card rectangles, pin anchors and canvas size.
pub(super) struct Layout<'a> {
    pub(super) components: HashMap<&'a str, [f64; 4]>,
    pub(super) ports: HashMap<&'a str, (f64, f64)>,
    pub(super) width: f64,
    pub(super) height: f64,
}

/// Placement cursor: next card position and the tallest card in the current row.
struct Cursor {
    x: f64,
    y: f64,
    row_h: f64,
}

/// Places components left to right in rows of at least three, wrapping by row height.
pub(super) fn layout<'a>(circuit: &'a Circuit, components: &[&'a Component]) -> Layout<'a> {
    let cols = 3.max((components.len() as f64).sqrt().ceil() as usize);
    let mut layout = Layout {
        components: HashMap::new(),
        ports: HashMap::new(),
        width: 0.0,
        height: 0.0,
    };
    let mut cursor = Cursor {
        x: ORIGIN,
        y: ORIGIN,
        row_h: 0.0,
    };
    for (idx, comp) in components.iter().enumerate() {
        layout.place(circuit, comp, &mut cursor);
        if (idx + 1) % cols == 0 {
            cursor.x = ORIGIN;
            cursor.y += cursor.row_h + 25.0;
            cursor.row_h = 0.0;
        }
    }
    layout.height = cursor.y + cursor.row_h + 30.0;
    layout
}

impl<'a> Layout<'a> {
    /// Places one card at the cursor, records its pin anchors and advances the cursor.
    fn place(&mut self, circuit: &'a Circuit, comp: &'a Component, cursor: &mut Cursor) {
        let cid = component_id(comp);
        let ports = component_ports(circuit, cid);
        let comp_h = 16.0 + (ports.len() as f64 * PIN_H).max(12.0);
        if comp_h > cursor.row_h {
            cursor.row_h = comp_h;
        }
        self.components
            .insert(cid, [cursor.x, cursor.y, COMP_W, comp_h]);
        for (p_idx, port) in ports.iter().enumerate() {
            let py = cursor.y + 16.0 + (p_idx as f64 * PIN_H);
            if let Some(pid) = port.port_id.as_deref() {
                self.ports.insert(pid, (cursor.x + COMP_W, py));
            }
        }
        cursor.x += COMP_W + GAP_X;
        if cursor.x > self.width {
            self.width = cursor.x;
        }
    }
}
