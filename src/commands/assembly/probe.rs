//! Probe solids: the board side of the cross-world rules, built from a board's kit.
//!
//! A probe is a small solid in the board frame plus the matrix that places it in the
//! assembly frame. Nothing here touches a file or a kernel; the measure script builds the
//! solids and reports how much of each one a body fills.

use serde::Serialize;

use super::place::Matrix;
use super::probe_holes::hole_probes;
use super::rules::{OPENING_DEPTH, UNMEASURED_HEIGHT};
use super::shrink::{shrunk_corners, shrunk_polygon, shrunk_span};
use crate::commands::body::{Kit, KitOpening};

/// The face of the board a hole probe sits against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Bottom,
    Top,
}

impl Side {
    /// The side as it reads in a message.
    pub fn label(self) -> &'static str {
        match self {
            Side::Bottom => "bottom",
            Side::Top => "top",
        }
    }
}

/// The geometry of a probe in the board frame, tagged by `kind` for the script.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Shape {
    Prism {
        points: Vec<[f64; 2]>,
        z0: f64,
        z1: f64,
    },
    Box {
        min: [f64; 3],
        max: [f64; 3],
    },
    Cylinder {
        center: [f64; 2],
        r: f64,
        z0: f64,
        z1: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        clip: Option<Vec<[f64; 2]>>,
    },
    Ring {
        center: [f64; 2],
        r0: f64,
        r1: f64,
        z0: f64,
        z1: f64,
        quadrant: u8,
    },
}

/// What a probe stands for, so the rules can read its measurement.
#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Slab,
    KeepOut {
        component: String,
    },
    Opening {
        component: String,
        side: &'static str,
    },
    HoleNear {
        hole: usize,
        side: Side,
        at: [f64; 2],
        plated: bool,
    },
    HoleQuadrant {
        hole: usize,
        side: Side,
    },
    HoleBore {
        hole: usize,
        at: [f64; 2],
    },
}

/// One probe: its id, what it stands for, its shape, and its placement.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Probe {
    pub id: String,
    #[serde(skip)]
    pub board: String,
    #[serde(skip)]
    pub role: Role,
    pub collide: bool,
    pub show: bool,
    #[serde(flatten)]
    pub shape: Shape,
    pub matrix: Matrix,
}

pub(super) struct Frame<'a> {
    board: &'a str,
    matrix: &'a Matrix,
}

impl Frame<'_> {
    pub(super) fn probe(&self, id: String, role: Role, shape: Shape) -> Probe {
        Probe {
            id: format!("{}:{id}", self.board),
            board: self.board.to_string(),
            collide: matches!(
                role,
                Role::Slab | Role::KeepOut { .. } | Role::Opening { .. } | Role::HoleBore { .. }
            ),
            show: role == Role::Slab,
            role,
            shape,
            matrix: *self.matrix,
        }
    }
}

fn slab(frame: &Frame, kit: &Kit) -> Probe {
    let half = kit.thickness / 2.0;
    let (z0, z1) = shrunk_span(-half, half);
    let shape = Shape::Prism {
        points: shrunk_polygon(&kit.outline),
        z0,
        z1,
    };
    frame.probe("slab".into(), Role::Slab, shape)
}

fn shrunk_box(min: [f64; 3], max: [f64; 3]) -> Shape {
    let (min, max) = shrunk_corners(min, max);
    Shape::Box { min, max }
}

fn keep_outs(frame: &Frame, kit: &Kit) -> Vec<Probe> {
    kit.keep_out
        .iter()
        .enumerate()
        .map(|(i, k)| {
            let min = [k.x - k.w / 2.0, k.y - k.h / 2.0, k.z0];
            let max = [k.x + k.w / 2.0, k.y + k.h / 2.0, k.z1];
            let role = Role::KeepOut {
                component: k.component.clone(),
            };
            frame.probe(format!("keep_out:{i}"), role, shrunk_box(min, max))
        })
        .collect()
}

fn opening_span(kit: &Kit, opening: &KitOpening) -> (f64, f64) {
    let half = kit.thickness / 2.0;
    let component = kit.components.iter().find(|c| c.name == opening.component);
    let measured = component.and_then(|c| Some((c.z?, c.z? + c.height?)));
    let on_bottom = component.is_some_and(|c| c.layer == "bottom");
    measured.unwrap_or(if on_bottom {
        (-half - UNMEASURED_HEIGHT, -half)
    } else {
        (half, half + UNMEASURED_HEIGHT)
    })
}

fn openings(frame: &Frame, kit: &Kit) -> Vec<Probe> {
    kit.openings
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let (z0, z1) = opening_span(kit, o);
            let (z0, z1) = shrunk_span(z0, z1);
            let len = o.normal[0].hypot(o.normal[1]);
            let out = o.normal.map(|n| n / len * OPENING_DEPTH);
            let push = |p: [f64; 2]| [p[0] + out[0], p[1] + out[1]];
            let quad = [o.from, o.to, push(o.to), push(o.from)];
            let shape = Shape::Prism {
                points: shrunk_polygon(&quad),
                z0,
                z1,
            };
            let role = Role::Opening {
                component: o.component.clone(),
                side: o.side,
            };
            frame.probe(format!("opening:{i}"), role, shape)
        })
        .collect()
}

/// Builds every probe of the board `board`, placed by `frame` in the assembly frame.
pub fn probes(board: &str, kit: &Kit, frame: &Matrix) -> Vec<Probe> {
    let frame = Frame {
        board,
        matrix: frame,
    };
    let mut all = vec![slab(&frame, kit)];
    all.extend(keep_outs(&frame, kit));
    all.extend(openings(&frame, kit));
    all.extend(hole_probes(&frame, kit));
    all
}
