//! Copper ring and drill geometry of plated and non-plated holes.

use super::elements::{PcbHoleElm, PcbPlatedHoleElm};
use super::geometry::{Drill, PadShape};
use super::math::{add, finite_or, required};

/// Returns the copper pad and the drill of a plated hole.
pub(super) fn plated_hole_geometry(hole: &PcbPlatedHoleElm) -> Result<(PadShape, Drill), String> {
    let center = [hole.x, hole.y];
    let drilled_at = offset_drill_center(hole, center)?;
    match hole.shape.as_deref() {
        Some("circle") => Ok((
            PadShape::Circle {
                d: required(hole.outer_diameter, "outer_diameter")?,
            },
            round(hole.hole_diameter, center)?,
        )),
        Some("oval") | Some("pill") => pill_hole(hole, center),
        Some("circular_hole_with_rect_pad") => Ok((
            rect_pad(
                hole,
                finite_or(hole.rect_ccw_rotation, 0.0, "rect_ccw_rotation")?,
            )?,
            round(hole.hole_diameter, drilled_at)?,
        )),
        Some("pill_hole_with_rect_pad") => Ok((
            rect_pad(hole, 0.0)?,
            slot(hole.hole_width, hole.hole_height, 0.0, drilled_at)?,
        )),
        Some("rotated_pill_hole_with_rect_pad") => rotated_pill_rect_hole(hole, drilled_at),
        Some(other) => Err(format!("unsupported plated hole shape '{other}'")),
        None => Err("missing shape".to_string()),
    }
}

/// Returns where the drill sits: the hole centre shifted by its optional offset.
fn offset_drill_center(hole: &PcbPlatedHoleElm, center: [f64; 2]) -> Result<[f64; 2], String> {
    let offset = [
        finite_or(hole.hole_offset_x, 0.0, "hole_offset_x")?,
        finite_or(hole.hole_offset_y, 0.0, "hole_offset_y")?,
    ];
    Ok(add(center, offset))
}

/// Returns the rotated rectangular pad and rotated slot of a `rotated_pill_hole_with_rect_pad`.
fn rotated_pill_rect_hole(
    hole: &PcbPlatedHoleElm,
    drilled_at: [f64; 2],
) -> Result<(PadShape, Drill), String> {
    Ok((
        rect_pad(hole, required(hole.rect_ccw_rotation, "rect_ccw_rotation")?)?,
        slot(
            hole.hole_width,
            hole.hole_height,
            required(hole.hole_ccw_rotation, "hole_ccw_rotation")?,
            drilled_at,
        )?,
    ))
}

/// Returns the obround pad and centred slot of an `oval` or `pill` plated hole.
fn pill_hole(hole: &PcbPlatedHoleElm, center: [f64; 2]) -> Result<(PadShape, Drill), String> {
    let rot = finite_or(hole.ccw_rotation, 0.0, "ccw_rotation")?;
    Ok((
        PadShape::Pill {
            w: required(hole.outer_width, "outer_width")?,
            h: required(hole.outer_height, "outer_height")?,
            rot,
        },
        slot(hole.hole_width, hole.hole_height, rot, center)?,
    ))
}

/// Returns the rectangular pad of a plated hole rotated by `rot`.
fn rect_pad(hole: &PcbPlatedHoleElm, rot: f64) -> Result<PadShape, String> {
    Ok(PadShape::Rect {
        w: required(hole.rect_pad_width, "rect_pad_width")?,
        h: required(hole.rect_pad_height, "rect_pad_height")?,
        rot,
    })
}

/// Returns a slot drill from its required width and height.
fn slot(w: Option<f64>, h: Option<f64>, rot: f64, at: [f64; 2]) -> Result<Drill, String> {
    Ok(Drill::Slot {
        w: required(w, "hole_width")?,
        h: required(h, "hole_height")?,
        rot,
        at,
    })
}

/// Returns a round drill from its required diameter.
fn round(d: Option<f64>, at: [f64; 2]) -> Result<Drill, String> {
    Ok(Drill::Round {
        d: required(d, "hole_diameter")?,
        at,
    })
}

/// Returns the drill of a non-plated `pcb_hole`.
pub(super) fn hole_drill(hole: &PcbHoleElm) -> Result<Drill, String> {
    let at = [hole.x, hole.y];
    match hole.hole_shape.as_deref() {
        Some("circle") => round(hole.hole_diameter, at),
        Some("oval") | Some("pill") => slot(
            hole.hole_width,
            hole.hole_height,
            finite_or(hole.ccw_rotation, 0.0, "ccw_rotation")?,
            at,
        ),
        Some("rotated_pill") => slot(
            hole.hole_width,
            hole.hole_height,
            required(hole.ccw_rotation, "ccw_rotation")?,
            at,
        ),
        Some(other) => Err(format!("unsupported hole shape '{other}'")),
        None => Err("missing hole_shape".to_string()),
    }
}
