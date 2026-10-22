//! Silkscreen text stroked with the built-in vector font.

use super::elements::PcbModel;
use super::elements::{PcbPoint, PcbSilkscreenTextElm};
use super::font::{glyph, ADVANCE, GLYPH_HEIGHT, GLYPH_WIDTH};
use super::geometry::Feature;
use super::math::{add, finite_or, required, rotate};
use super::{invalid, GerberError};
use crate::gerber::apertures::Aperture;

const MIN_STROKE_MM: f64 = 0.1;
const STROKE_RATIO: f64 = 0.12;

impl PcbModel {
    /// Returns the strokes of every silkscreen text on one side; untagged texts belong to the top.
    pub(super) fn silkscreen_text_features(
        &self,
        is_top: bool,
    ) -> Result<Vec<Feature>, GerberError> {
        let mut features = Vec::new();
        for (i, text) in self.silkscreen_texts.iter().enumerate() {
            if text_on_top(text).map_err(invalid("pcb_silkscreen_text", i))? == is_top {
                features
                    .extend(text_strokes(text, is_top).map_err(invalid("pcb_silkscreen_text", i))?);
            }
        }
        Ok(features)
    }
}

/// Returns true when the text sits on the top side.
fn text_on_top(text: &PcbSilkscreenTextElm) -> Result<bool, String> {
    match text.layer.as_deref() {
        Some("top") | None => Ok(true),
        Some("bottom") => Ok(false),
        Some(other) => Err(format!("unsupported silkscreen layer '{other}'")),
    }
}

/// Returns the strokes of one text, rotated about its anchor and mirrored on the bottom side.
fn text_strokes(text: &PcbSilkscreenTextElm, is_top: bool) -> Result<Vec<Feature>, String> {
    let content = text.text.as_deref().ok_or("missing text")?;
    let size = required(text.font_size, "font_size")?;
    let rot = finite_or(text.ccw_rotation, 0.0, "ccw_rotation")?;
    let unit = size / GLYPH_HEIGHT;
    let aperture = Aperture::circle((size * STROKE_RATIO).max(MIN_STROKE_MM))?;
    let place = Placement {
        anchor: anchor_point(text)?,
        origin: box_origin(text.anchor_alignment.as_deref(), content, unit)?,
        unit,
        rot,
        mirror: !is_top,
    };
    let mut features = Vec::new();
    for (index, ch) in content.chars().enumerate() {
        let strokes = glyph(ch).ok_or_else(|| format!("character '{ch}' has no glyph"))?;
        for stroke in strokes {
            let pts: Vec<[f64; 2]> = stroke.iter().map(|g| place.point(*g, index)).collect();
            features.extend(
                pts.windows(2)
                    .map(|w| Feature::Stroke(aperture.clone(), w[0], w[1])),
            );
        }
    }
    Ok(features)
}

struct Placement {
    anchor: [f64; 2],
    origin: [f64; 2],
    unit: f64,
    rot: f64,
    mirror: bool,
}

impl Placement {
    /// Maps a glyph grid point of the `index`-th character to board coordinates.
    fn point(&self, [gx, gy]: [f64; 2], index: usize) -> [f64; 2] {
        let x = (gx + ADVANCE * index as f64) * self.unit - self.origin[0];
        let local = [
            if self.mirror { -x } else { x },
            gy * self.unit - self.origin[1],
        ];
        add(self.anchor, rotate(local, self.rot))
    }
}

/// Returns the anchor position, falling back to `center`.
fn anchor_point(text: &PcbSilkscreenTextElm) -> Result<[f64; 2], String> {
    let PcbPoint { x, y } = text
        .anchor_position
        .or(text.center)
        .ok_or("missing anchor_position")?;
    Ok([
        required(Some(x), "anchor x")?,
        required(Some(y), "anchor y")?,
    ])
}

/// Returns the offset from the text box's bottom-left corner to the point sitting on the anchor.
fn box_origin(alignment: Option<&str>, content: &str, unit: f64) -> Result<[f64; 2], String> {
    let count = content.chars().count() as f64;
    let width = ((count - 1.0) * ADVANCE + GLYPH_WIDTH).max(0.0) * unit;
    let height = GLYPH_HEIGHT * unit;
    let (vertical, horizontal) = match alignment.unwrap_or("center") {
        "center" => ("center", "center"),
        other => other
            .split_once('_')
            .ok_or_else(|| format!("unsupported anchor_alignment '{other}'"))?,
    };
    let x = match horizontal {
        "left" => 0.0,
        "center" => width / 2.0,
        "right" => width,
        other => return Err(format!("unsupported anchor_alignment '{other}'")),
    };
    let y = match vertical {
        "bottom" => 0.0,
        "center" => height / 2.0,
        "top" => height,
        other => return Err(format!("unsupported anchor_alignment '{other}'")),
    };
    Ok([x, y])
}
