//! Board outline derivation and the Edge Cuts profile layer.

use super::elements::{PcbBoardElm, PcbModel, PcbPoint};
use super::math::{coord, required};
use super::{invalid, GerberError};
use crate::gerber::apertures::{Aperture, ApertureDictionary};
use crate::gerber::coordinates::Coord;
use crate::gerber::writer::{FilePolarity, GerberWriter};

const EDGE_STROKE_NM: i64 = 150_000;

impl PcbBoardElm {
    /// Returns the closed board outline, deriving a rectangle from `width`, `height`,
    /// and `center` when no explicit outline is set.
    pub fn outline_points(&self) -> Vec<PcbPoint> {
        if let Some(outline) = &self.outline {
            return outline.clone();
        }
        let w = self.width.unwrap_or(80.0) / 2.0;
        let h = self.height.unwrap_or(60.0) / 2.0;
        let cx = self.center.as_ref().map(|c| c.x).unwrap_or(0.0);
        let cy = self.center.as_ref().map(|c| c.y).unwrap_or(0.0);
        [(-w, -h), (w, -h), (w, h), (-w, h), (-w, -h)]
            .into_iter()
            .map(|(dx, dy)| PcbPoint {
                x: cx + dx,
                y: cy + dy,
            })
            .collect()
    }
}

impl PcbModel {
    /// Builds the board profile, closing every outline back to its first vertex.
    pub fn build_edge_cuts(&self) -> Result<String, GerberError> {
        let mut rings = Vec::new();
        for (i, board) in self.boards.iter().enumerate() {
            rings.push(closed_outline(board).map_err(invalid("pcb_board", i))?);
        }

        let mut dict = ApertureDictionary::new();
        let border_ap = dict.intern(Aperture::Circle {
            diameter_nm: EDGE_STROKE_NM,
        });
        let mut writer = GerberWriter::new(
            "Edge Cuts (Board Outline)",
            "Profile,NP",
            FilePolarity::Positive,
            &dict,
        );
        writer.select_aperture(border_ap);
        for ring in rings {
            let [fx, fy] = ring[0];
            writer.move_to(fx, fy);
            for &[x, y] in &ring[1..] {
                writer.draw_to(x, y);
            }
        }
        Ok(writer.finish())
    }
}

/// Rejects boards whose outline or rectangle dimensions cannot form a profile.
fn check_outline_source(board: &PcbBoardElm) -> Result<(), String> {
    match &board.outline {
        Some(points) if points.len() < 3 => Err(format!(
            "outline has {} point(s), needs at least 3",
            points.len()
        )),
        Some(_) => Ok(()),
        None => {
            if required(board.width, "width")? <= 0.0 || required(board.height, "height")? <= 0.0 {
                return Err("width and height must be positive".to_string());
            }
            Ok(())
        }
    }
}

/// Returns the board outline as a closed ring of fixed-point coordinates.
fn closed_outline(board: &PcbBoardElm) -> Result<Vec<[Coord; 2]>, String> {
    check_outline_source(board)?;
    let points = board.outline_points();
    if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
        return Err("outline contains a non-finite coordinate".to_string());
    }
    let mut ring: Vec<[Coord; 2]> = points.iter().map(|p| coord([p.x, p.y])).collect();
    if ring.first() != ring.last() {
        ring.push(ring[0]);
    }
    Ok(ring)
}
