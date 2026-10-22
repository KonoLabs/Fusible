//! The JSON spec handed to the generator, built from the parsed board.

use std::path::PathBuf;

use serde::Serialize;

use super::layout::{notches, vents};
use super::settings::Settings;
use super::{
    MechanicalError, MechanicalPreview, BOSS_WALL, CLEARANCE, DEFAULT_BOARD_THICKNESS, HEADROOM,
    PILOT_UNDERSIZE, STANDOFF,
};
use crate::gerber::layers::{PcbHoleElm, PcbPoint};
use crate::gerber::PcbModel;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Bounds {
    pub(super) min_x: f64,
    pub(super) min_y: f64,
    pub(super) max_x: f64,
    pub(super) max_y: f64,
}

#[derive(Debug, Serialize, PartialEq)]
pub(super) struct Boss {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) outer: f64,
    pub(super) pilot: f64,
}

#[derive(Debug, Serialize, PartialEq)]
pub(super) struct Notch {
    pub(super) points: [[f64; 2]; 4],
}

#[derive(Debug, Serialize, PartialEq)]
pub(super) struct Vent {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) w: f64,
    pub(super) h: f64,
    pub(super) round: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct Output {
    pub(super) format: &'static str,
    pub(super) path: PathBuf,
}

#[derive(Debug, Serialize)]
pub(super) struct Spec {
    pub(super) outline: Vec<[f64; 2]>,
    pub(super) clearance: f64,
    pub(super) wall: f64,
    pub(super) height: f64,
    pub(super) board_z: f64,
    pub(super) board_thickness: f64,
    pub(super) bosses: Vec<Boss>,
    pub(super) notches: Vec<Notch>,
    pub(super) vents: Vec<Vent>,
    pub(super) outputs: Vec<Output>,
}

/// Lays out the whole enclosure (outline, bosses, notches, vents) for the generator.
pub(super) fn build_spec(
    model: &PcbModel,
    settings: &Settings,
    outputs: Vec<Output>,
) -> Result<Spec, MechanicalError> {
    let board = model.boards.first().ok_or(MechanicalError::NoBoard)?;
    let outline = open_ring(board.outline_points());
    if outline.len() < 3 {
        return Err(MechanicalError::NoBoard);
    }
    let board_thickness = board.thickness.unwrap_or(DEFAULT_BOARD_THICKNESS);
    let bosses = if settings.mounting {
        bosses(&model.holes)
    } else {
        Vec::new()
    };
    let board_z = settings.wall + if bosses.is_empty() { 0.0 } else { STANDOFF };
    let height = stack_height(settings.height, board_z + board_thickness)?;
    let notches = match settings.cutouts {
        true => notches(&model.components, &outline, settings.wall),
        false => Vec::new(),
    };
    Ok(Spec {
        vents: vents(settings.ventilation, &outline, bounds_of(&outline)),
        outline: outline.iter().map(|p| [p.x, p.y]).collect(),
        clearance: CLEARANCE,
        wall: settings.wall,
        height,
        board_z,
        board_thickness,
        bosses,
        notches,
        outputs,
    })
}

/// Derives the studio preview metadata (frame and outer size) from a spec.
pub(super) fn preview_from(spec: &Spec, glb: Vec<u8>) -> MechanicalPreview {
    let points: Vec<PcbPoint> = spec
        .outline
        .iter()
        .map(|&[x, y]| PcbPoint { x, y })
        .collect();
    let b = bounds_of(&points);
    let margin = 2.0 * (spec.clearance + spec.wall);
    MechanicalPreview {
        glb,
        board_z: spec.board_z,
        board_thickness: spec.board_thickness,
        height: spec.height,
        wall: spec.wall,
        size: [
            b.max_x - b.min_x + margin,
            b.max_y - b.min_y + margin,
            spec.height + spec.wall,
        ],
    }
}

fn stack_height(requested: Option<f64>, minimum: f64) -> Result<f64, MechanicalError> {
    let height = requested.unwrap_or(minimum + HEADROOM);
    if height.is_finite() && height > minimum {
        return Ok(height);
    }
    Err(MechanicalError::InvalidDimension {
        name: "height",
        value: height,
        help: format!("the board stack needs more than {minimum:.2} mm"),
    })
}

/// Drops the closing point of a ring whose last point repeats the first.
pub(super) fn open_ring(mut points: Vec<PcbPoint>) -> Vec<PcbPoint> {
    if let (Some(first), Some(last)) = (points.first(), points.last()) {
        if points.len() > 1 && first.x == last.x && first.y == last.y {
            points.pop();
        }
    }
    points
}

/// Returns the axis-aligned bounds of `points`.
pub(super) fn bounds_of(points: &[PcbPoint]) -> Bounds {
    points.iter().fold(
        Bounds {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
        },
        |b, p| Bounds {
            min_x: b.min_x.min(p.x),
            min_y: b.min_y.min(p.y),
            max_x: b.max_x.max(p.x),
            max_y: b.max_y.max(p.y),
        },
    )
}

fn bosses(holes: &[PcbHoleElm]) -> Vec<Boss> {
    holes
        .iter()
        .filter_map(|h| {
            let d = h.hole_diameter.filter(|d| *d > 0.0)?;
            Some(Boss {
                x: h.x,
                y: h.y,
                outer: d + 2.0 * BOSS_WALL,
                pilot: (d - PILOT_UNDERSIZE).max(d * 0.5),
            })
        })
        .collect()
}
