//! SMT pad shapes and trace segment layer resolution.

use super::copper::CopperLayer;
use super::elements::{PcbSmtPadElm, PcbTracePoint};
use super::geometry::PadShape;
use super::math::{finite_or, required};
use super::sites::TraceSegment;

/// Returns the copper shape of an SMT pad.
pub(super) fn smtpad_shape(pad: &PcbSmtPadElm) -> Result<PadShape, String> {
    let rot = finite_or(pad.ccw_rotation, 0.0, "ccw_rotation")?;
    let w = || required(pad.width, "width");
    let h = || required(pad.height, "height");
    let shape = pad.shape.as_deref();
    if shape != Some("polygon") {
        required(Some(pad.x), "x")?;
        required(Some(pad.y), "y")?;
    }
    match shape {
        Some("rect") => rect_shape(pad, rot),
        Some("rotated_rect") => rect_shape(pad, required(pad.ccw_rotation, "ccw_rotation")?),
        Some("polygon") => polygon_shape(pad),
        Some("circle") => Ok(PadShape::Circle {
            d: required(pad.radius, "radius")? * 2.0,
        }),
        Some("pill") | Some("oval") => Ok(PadShape::Pill {
            w: w()?,
            h: h()?,
            rot,
        }),
        Some("rotated_pill") => Ok(PadShape::Pill {
            w: w()?,
            h: h()?,
            rot: required(pad.ccw_rotation, "ccw_rotation")?,
        }),
        Some(other) => Err(format!("unsupported pad shape '{other}'")),
        None => Err("missing shape".to_string()),
    }
}

/// Returns a rectangle pad, rounded when `rect_border_radius` or `corner_radius` is positive.
fn rect_shape(pad: &PcbSmtPadElm, rot: f64) -> Result<PadShape, String> {
    let (w, h) = (
        required(pad.width, "width")?,
        required(pad.height, "height")?,
    );
    let radius = pad.rect_border_radius.or(pad.corner_radius);
    let r = finite_or(radius, 0.0, "rect_border_radius")?;
    if r < 0.0 {
        return Err(format!("rect_border_radius {r} must not be negative"));
    }
    Ok(if r > 0.0 {
        PadShape::RoundRect { w, h, r, rot }
    } else {
        PadShape::Rect { w, h, rot }
    })
}

/// Returns a polygon pad from its absolute `points`, requiring at least three finite vertices.
fn polygon_shape(pad: &PcbSmtPadElm) -> Result<PadShape, String> {
    let points = pad.points.as_deref().unwrap_or_default();
    if points.len() < 3 {
        return Err(format!(
            "polygon needs at least 3 points, got {}",
            points.len()
        ));
    }
    for p in points {
        required(Some(p.x), "polygon point x")?;
        required(Some(p.y), "polygon point y")?;
    }
    Ok(PadShape::Polygon {
        points: points.iter().map(|p| [p.x, p.y]).collect(),
    })
}

/// Returns the layer and width of a wire route point, or `None` for a via.
fn route_wire(p: &PcbTracePoint) -> Result<Option<(CopperLayer, f64)>, String> {
    match p.route_type.as_deref() {
        Some("wire") => Ok(Some((
            CopperLayer::parse(p.layer.as_deref())?,
            required(p.width, "wire width")?,
        ))),
        Some("via") => Ok(None),
        Some(other) => Err(format!("unsupported route_type '{other}'")),
        None => Err("route point missing route_type".to_string()),
    }
}

/// Returns the straight trace segment between two consecutive route points.
pub(super) fn trace_segment(
    p1: &PcbTracePoint,
    p2: &PcbTracePoint,
) -> Result<TraceSegment, String> {
    let (layer, width) = match (route_wire(p1)?, route_wire(p2)?) {
        (Some((l1, w)), Some((l2, _))) if l1 == l2 => (l1, w),
        (Some(_), Some(_)) => return Err("wire changes layer without a via".to_string()),
        (Some(a), None) | (None, Some(a)) => a,
        (None, None) => return Err("segment between two vias has no layer".to_string()),
    };
    Ok(TraceSegment {
        layer,
        width,
        from: [p1.x, p1.y],
        to: [p2.x, p2.y],
    })
}
