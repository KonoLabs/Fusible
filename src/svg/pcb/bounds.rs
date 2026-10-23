//! Screen-space bounding box of everything drawn on the PCB.

use crate::gerber::layers::{PcbBoardElm, PcbModel, PcbSmtPadElm};

/// Axis-aligned bounds in screen space (Y flipped).
#[derive(Debug, Clone, Copy)]
pub(super) struct Bounds {
    pub(super) min_x: f64,
    pub(super) min_y: f64,
    pub(super) max_x: f64,
    pub(super) max_y: f64,
}

impl Bounds {
    /// Grows the bounds to contain `(x, y)`.
    fn include(&mut self, x: f64, y: f64) {
        if x < self.min_x {
            self.min_x = x;
        }
        if y < self.min_y {
            self.min_y = y;
        }
        if x > self.max_x {
            self.max_x = x;
        }
        if y > self.max_y {
            self.max_y = y;
        }
    }
}

/// Returns the bounds of board outlines, pads, plated holes and vias, or a
/// default 50 x 50 mm box when the model draws nothing.
pub(super) fn calculate_bounds(model: &PcbModel) -> Bounds {
    let mut bounds = Bounds {
        min_x: f64::MAX,
        min_y: f64::MAX,
        max_x: f64::MIN,
        max_y: f64::MIN,
    };
    let outline = model.boards.iter().flat_map(PcbBoardElm::outline_points);
    let points = outline
        .map(|pt| (pt.x, pt.y))
        .chain(model.smt_pads.iter().flat_map(pad_points))
        .chain(model.plated_holes.iter().map(|h| (h.x, h.y)))
        .chain(model.vias.iter().map(|v| (v.x, v.y)));
    for (x, y) in points {
        bounds.include(x, -y);
    }
    if bounds.min_x > bounds.max_x {
        return Bounds {
            min_x: -25.0,
            min_y: -25.0,
            max_x: 25.0,
            max_y: 25.0,
        };
    }
    bounds
}

/// The points a pad occupies: its polygon vertices, or its center.
fn pad_points(pad: &PcbSmtPadElm) -> Vec<(f64, f64)> {
    match pad.points.as_deref() {
        Some(points) if pad.shape.as_deref() == Some("polygon") => {
            points.iter().map(|pt| (pt.x, pt.y)).collect()
        }
        _ => vec![(pad.x, pad.y)],
    }
}
