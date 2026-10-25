//! The hole rule: a boss rings the hole and nothing fills its bore.

use std::collections::{HashMap, HashSet};

use super::probe::{Probe, Role, Side};
use super::rules::{solid, Measurement, Violation, HOLE_MISALIGNED};

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
        Role::HoleNear { hole, side, at, .. } => {
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

fn near_bodies<'a>(
    probes: &'a [Probe],
    hits: &'a [Measurement],
) -> HashSet<(&'a str, usize, &'a str)> {
    let roles: HashMap<&str, &Probe> = probes.iter().map(|p| (p.id.as_str(), p)).collect();
    hits.iter()
        .filter(|h| solid(h.volume))
        .filter_map(|h| {
            let probe = roles.get(h.probe.as_str())?;
            match probe.role {
                Role::HoleNear { hole, .. } => Some((probe.board.as_str(), hole, h.body.as_str())),
                _ => None,
            }
        })
        .collect()
}

fn no_boss(body: &str, board: &str, at: [f64; 2]) -> Violation {
    Violation {
        rule: HOLE_MISALIGNED,
        message: format!(
            "body `{body}` has no boss for the hole at ({:.2}, {:.2}) of board `{board}`",
            at[0], at[1]
        ),
        parts: vec![body.to_string(), board.to_string()],
        volume: None,
        at: None,
        collision: None,
    }
}

/// One violation per non-plated hole of a fitted board that has no body material near it.
pub(super) fn missing_bosses(
    probes: &[Probe],
    hits: &[Measurement],
    fitted: &[(String, String)],
) -> Vec<Violation> {
    let near = near_bodies(probes, hits);
    let plain = |probe: &Probe| match probe.role {
        Role::HoleNear {
            hole,
            side: Side::Top,
            at,
            plated: false,
        } => Some((hole, at)),
        _ => None,
    };
    let mut found = Vec::new();
    for (body, board) in fitted {
        let holes = probes.iter().filter(|p| &p.board == board);
        for (hole, at) in holes.filter_map(plain) {
            if !near.contains(&(board.as_str(), hole, body.as_str())) {
                found.push(no_boss(body, board, at));
            }
        }
    }
    found
}
