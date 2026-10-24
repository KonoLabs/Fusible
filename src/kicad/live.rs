//! One pass of the KiCad sync: compare a board file with the Circuit JSON, then write positions.

use std::path::Path;

use super::changes::design_differences;
use super::diff::{compare_kicad_with_circuit, parse_circuit_elements, DiffReport, PlacementDiff};
use super::sexpr::{extract_footprints, parse_sexpr};
use super::tsx::rewrite_tree;

/// A KiCad board compared with the design.
pub struct Comparison {
    report: DiffReport,
    /// Parts added, removed or renamed and nets rewired in KiCad; reported, never written.
    pub differences: Vec<String>,
}

impl Comparison {
    /// Designators whose placement KiCad changed.
    pub fn moved(&self) -> Vec<String> {
        self.report
            .diffs
            .iter()
            .filter(|d| d.is_placement_change())
            .map(|d| d.designator.clone())
            .collect()
    }
}

/// Compares the board file at `pcb` with `circuit_json`.
///
/// Fails when either side cannot be parsed; unparseable Circuit JSON is never read as an
/// empty circuit.
pub fn compare_board(pcb: &Path, circuit_json: &str) -> Result<Comparison, String> {
    let content =
        std::fs::read_to_string(pcb).map_err(|e| format!("cannot read {}: {e}", pcb.display()))?;
    let sexpr = parse_sexpr(&content).map_err(|e| format!("invalid KiCad file: {e}"))?;
    let footprints = extract_footprints(&sexpr);
    let elements =
        parse_circuit_elements(circuit_json).map_err(|e| format!("invalid Circuit JSON: {e}"))?;
    let report = compare_kicad_with_circuit(&footprints, &elements);
    let differences = design_differences(&report, &footprints, &elements);
    Ok(Comparison {
        report,
        differences,
    })
}

/// Writes the parts KiCad moved into the TSX files under `dir`, after copying each file it
/// changes to `backup_dir`.
///
/// Only plain-number `pcbX`, `pcbY` and `pcbRotation` props are rewritten. Returns the
/// designators written; the other moved parts stay different until the TSX is edited.
pub fn sync_positions(
    comparison: &Comparison,
    dir: &Path,
    backup_dir: &Path,
    name: &str,
) -> Result<Vec<String>, String> {
    let moved: Vec<&PlacementDiff> = comparison
        .report
        .diffs
        .iter()
        .filter(|d| d.is_placement_change())
        .collect();
    match moved.is_empty() {
        true => Ok(Vec::new()),
        false => rewrite_tree(dir, &moved, backup_dir, name),
    }
}
