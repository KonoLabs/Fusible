//! Pad and drill primitives and their rendering into Gerber features.

use super::math::{add, coord, long_axis, quarter_turns, rotate};
use super::rounded::{offset_polygon, round_rect_feature};
use crate::gerber::apertures::{Aperture, ApertureDictionary};
use crate::gerber::coordinates::Coord;
use crate::gerber::writer::{FilePolarity, GerberWriter};

#[derive(Debug, Clone)]
pub(super) enum PadShape {
    Circle { d: f64 },
    Rect { w: f64, h: f64, rot: f64 },
    RoundRect { w: f64, h: f64, r: f64, rot: f64 },
    Polygon { points: Vec<[f64; 2]> },
    Pill { w: f64, h: f64, rot: f64 },
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Drill {
    Round {
        d: f64,
        at: [f64; 2],
    },
    Slot {
        w: f64,
        h: f64,
        rot: f64,
        at: [f64; 2],
    },
}

#[derive(Debug)]
pub(super) enum Feature {
    Flash(Aperture, [f64; 2]),
    Stroke(Aperture, [f64; 2], [f64; 2]),
    Region(Vec<[f64; 2]>),
}

impl PadShape {
    /// Returns the feature drawing this pad at `at`, grown by `margin` on every side.
    pub(super) fn feature(self, at: [f64; 2], margin: f64) -> Result<Feature, String> {
        let grow = 2.0 * margin;
        match self {
            Self::Circle { d } => Ok(Feature::Flash(Aperture::circle(d + grow)?, at)),
            Self::Rect { w, h, rot } => rect_feature(w + grow, h + grow, rot, at),
            Self::Pill { w, h, rot } => pill_feature(w + grow, h + grow, rot, at),
            Self::RoundRect { w, h, r, rot } => {
                round_rect_feature(w + grow, h + grow, r + margin, rot, at)
            }
            Self::Polygon { points } => Ok(Feature::Region(offset_polygon(&points, margin)?)),
        }
    }
}

/// Returns the axis-aligned width and height after `rot`, or `None` for an arbitrary angle.
pub(super) fn axis_aligned(w: f64, h: f64, rot: f64) -> Option<(f64, f64)> {
    quarter_turns(rot).map(|q| if q % 2 == 1 { (h, w) } else { (w, h) })
}

/// Flashes an axis-aligned rectangle, or fills a rotated one as a region.
pub(super) fn rect_feature(w: f64, h: f64, rot: f64, at: [f64; 2]) -> Result<Feature, String> {
    if let Some((w, h)) = axis_aligned(w, h, rot) {
        return Ok(Feature::Flash(Aperture::rectangle(w, h)?, at));
    }
    Aperture::rectangle(w, h)?;
    let corners = [[-w, -h], [w, -h], [w, h], [-w, h]]
        .map(|[cx, cy]| add(at, rotate([cx / 2.0, cy / 2.0], rot)));
    Ok(Feature::Region(corners.to_vec()))
}

/// Flashes an axis-aligned obround, or strokes a rotated one along its long axis.
pub(super) fn pill_feature(w: f64, h: f64, rot: f64, at: [f64; 2]) -> Result<Feature, String> {
    if let Some((w, h)) = axis_aligned(w, h, rot) {
        return Ok(Feature::Flash(Aperture::obround(w, h)?, at));
    }
    Aperture::obround(w, h)?;
    let (from, to) = long_axis(w, h, rot, at);
    Ok(Feature::Stroke(Aperture::circle(w.min(h))?, from, to))
}

/// Renders features into a complete Gerber file with a deduplicated aperture dictionary.
pub(super) fn render(
    title: &str,
    func: &str,
    polarity: FilePolarity,
    features: &[Feature],
) -> String {
    let mut dict = ApertureDictionary::new();
    let ids: Vec<Option<u32>> = features
        .iter()
        .map(|f| match f {
            Feature::Flash(ap, _) | Feature::Stroke(ap, _, _) => Some(dict.intern(ap.clone())),
            Feature::Region(_) => None,
        })
        .collect();
    let mut writer = GerberWriter::new(title, func, polarity, &dict);
    for (feature, id) in features.iter().zip(ids) {
        if let Some(id) = id {
            writer.select_aperture(id);
        }
        match feature {
            Feature::Flash(_, [x, y]) => writer.flash(Coord::from_mm(*x), Coord::from_mm(*y)),
            Feature::Stroke(_, [x1, y1], [x2, y2]) => {
                writer.move_to(Coord::from_mm(*x1), Coord::from_mm(*y1));
                writer.draw_to(Coord::from_mm(*x2), Coord::from_mm(*y2));
            }
            Feature::Region(points) => {
                let pts: Vec<[Coord; 2]> = points.iter().map(|p| coord(*p)).collect();
                writer.region(&pts);
            }
        }
    }
    writer.finish()
}
