//! One pass of the KiCad sync: compare a board file with the Circuit JSON, then write positions.

use std::path::Path;

use super::backup::backup_file;
use super::changes::design_differences;
use super::diff::{compare_kicad_with_circuit, parse_circuit_elements, DiffReport};
use super::sexpr::{extract_footprints, parse_sexpr};
use super::sync::apply_kicad_sync;

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

/// Writes the moved parts into `edits`, after copying its previous content to `backup_dir`.
///
/// Returns the designators written; nothing is touched when no part moved.
pub fn sync_positions(
    comparison: &Comparison,
    edits: &Path,
    backup_dir: &Path,
    name: &str,
) -> Result<Vec<String>, String> {
    let moved = comparison.moved();
    if moved.is_empty() {
        return Ok(moved);
    }
    backup_file(edits, backup_dir, name)?;
    apply_kicad_sync(&comparison.report, edits, false)?;
    Ok(moved)
}
