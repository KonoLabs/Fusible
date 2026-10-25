//! The hole rule: a boss rings the hole and nothing fills its bore.

use std::collections::HashMap;

use super::probe::{Probe, Role, Side};
use super::rules::{solid, Measurement, HOLE_MISALIGNED};

/// Fewest ring quadrants that must hold material for a boss to count as present.
pub const MIN_BOSS_QUADRANTS: usize = 2;

pub(super) type Group = (String, usize, Side, String);

pub(super) fn aligned_quadrants(
    roles: &HashMap<&str, &Probe>,
    hits: &[Measurement],
) -> HashMap<Group, usize> {
    let mut counts = HashMap::new();
    for hit in hits.iter().filter(|h| solid(h.volume)) {
        let Some(probe) = roles.get(hit.probe.as_str()) else {
            continue;
        };
        if let Role::HoleQuadrant { hole, side } = &probe.role {
            let key = (probe.board.clone(), *hole, *side, hit.body.clone());
            *counts.entry(key).or_insert(0) += 1;
        }
    }
    counts
}

fn hole_message(probe: &Probe, body: &str, at: [f64; 2], side: Side) -> String {
    format!(
        "body `{body}` has material near the hole at ({:.2}, {:.2}) on the {} side of board `{}`, \
         but no boss rings the hole",
        at[0],
        at[1],
        side.label(),
        probe.board
    )
}

pub(super) fn hole_finding(
    probe: &Probe,
    hit: &Measurement,
    quadrants: &HashMap<Group, usize>,
) -> Option<(&'static str, String)> {
    let (body, board) = (hit.body.as_str(), probe.board.as_str());
    match &probe.role {
        Role::HoleNear { hole, side, at } => {
            let key = (probe.board.clone(), *hole, *side, hit.body.clone());
            if quadrants.get(&key).copied().unwrap_or(0) >= MIN_BOSS_QUADRANTS {
                return None;
            }
            Some((HOLE_MISALIGNED, hole_message(probe, body, *at, *side)))
        }
        Role::HoleBore { at, .. } => Some((
            HOLE_MISALIGNED,
            format!(
                "body `{body}` has material inside the hole at ({:.2}, {:.2}) of board `{board}`",
                at[0], at[1]
            ),
        )),
        _ => None,
    }
}
