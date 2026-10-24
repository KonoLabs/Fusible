//! Changes KiCad makes to the design that are reported and never written back.

use std::collections::BTreeSet;

use serde_json::Value;

use super::diff::{DiffKind, DiffReport, PlacementDiff};
use super::nets::net_differences;
use super::sexpr::KicadFootprint;

const SAME_SPOT_MM: f64 = 0.5;

/// Lists what KiCad added, removed, renamed or rewired compared with the design.
///
/// Moves, rotations and flips are not listed: they are the placement data `fusible sync`
/// writes. Each line is plain text for the studio notice.
pub fn design_differences(
    report: &DiffReport,
    kicad_fps: &[KicadFootprint],
    elements: &[Value],
) -> Vec<String> {
    let added: Vec<&PlacementDiff> = of_kind(report, &DiffKind::AddedInKicad);
    let missing: Vec<&PlacementDiff> = of_kind(report, &DiffKind::MissingInKicad);
    let renames = pair_renames(&added, &missing);
    let mut lines: Vec<String> = renames
        .iter()
        .map(|(from, to)| format!("{from} renamed to {to} in KiCad"))
        .collect();
    let renamed: BTreeSet<&str> = renames
        .iter()
        .flat_map(|(from, to)| [from.as_str(), to.as_str()])
        .collect();
    for diff in added
        .iter()
        .filter(|d| !renamed.contains(d.designator.as_str()))
    {
        lines.push(format!("{} added in KiCad", diff.designator));
    }
    for diff in missing
        .iter()
        .filter(|d| !renamed.contains(d.designator.as_str()))
    {
        lines.push(format!("{} removed in KiCad", diff.designator));
    }
    let shared = shared_designators(report);
    lines.extend(net_differences(kicad_fps, elements, &shared));
    lines
}

fn of_kind<'a>(report: &'a DiffReport, kind: &DiffKind) -> Vec<&'a PlacementDiff> {
    report.diffs.iter().filter(|d| &d.kind == kind).collect()
}

fn shared_designators(report: &DiffReport) -> BTreeSet<String> {
    report
        .diffs
        .iter()
        .filter(|d| d.kicad_pos.is_some() && d.current_pos.is_some())
        .map(|d| d.designator.clone())
        .collect()
}

/// Pairs each part only KiCad has with a part only the design has at the same spot.
fn pair_renames(added: &[&PlacementDiff], missing: &[&PlacementDiff]) -> Vec<(String, String)> {
    let mut free: Vec<&&PlacementDiff> = missing.iter().collect();
    let mut pairs = Vec::new();
    for new in added {
        let at = |d: &PlacementDiff| match (d.kicad_pos, d.current_pos) {
            (Some(p), _) | (None, Some(p)) => p,
            _ => [f64::MAX, f64::MAX],
        };
        let spot = at(new);
        let found = free.iter().position(|old| {
            let other = at(old);
            (other[0] - spot[0]).abs() < SAME_SPOT_MM && (other[1] - spot[1]).abs() < SAME_SPOT_MM
        });
        if let Some(i) = found {
            pairs.push((free.remove(i).designator.clone(), new.designator.clone()));
        }
    }
    pairs
}
