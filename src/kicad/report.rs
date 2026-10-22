//! Human-readable rendering of a placement [`DiffReport`].

use super::diff::{DiffKind, DiffReport, PlacementDiff};

pub fn format_diff_report(report: &DiffReport) -> String {
    let mut out = String::new();
    out.push_str("Component Layout Diff (KiCad <-> tscircuit):\n");
    for d in &report.diffs {
        out.push_str(&diff_entry(d));
    }
    out.push_str(&format!(
        "\nSummary: {} modified, {} unmatched, {} unchanged.\n",
        report.modified_count, report.unmatched_count, report.unchanged_count
    ));
    out
}

/// Returns the report lines for one diff, empty when it is unchanged.
fn diff_entry(d: &PlacementDiff) -> String {
    match &d.kind {
        DiffKind::Moved { dx, dy } => moved_entry(d, *dx, *dy),
        DiffKind::Rotated { d_rot } => format!(
            "  {} (Rotated):\n    rot: {:>5.1}° -> {:>5.1}° ({:+.1}°)\n",
            d.designator, d.current_rot, d.kicad_rot, d_rot
        ),
        DiffKind::Flipped { from, to } => format!(
            "  {} (Flipped):\n    layer: {} -> {}\n",
            d.designator, from, to
        ),
        DiffKind::AddedInKicad => {
            let k = d.kicad_pos.unwrap_or([0.0, 0.0]);
            format!(
                "  {} (Added in KiCad at ({:.2}, {:.2}))\n",
                d.designator, k[0], k[1]
            )
        }
        DiffKind::MissingInKicad => format!("  {} (Missing in KiCad)\n", d.designator),
        DiffKind::Unchanged => String::new(),
    }
}

fn moved_entry(d: &PlacementDiff, dx: f64, dy: f64) -> String {
    let cur = d.current_pos.unwrap_or([0.0, 0.0]);
    let k = d.kicad_pos.unwrap_or([0.0, 0.0]);
    format!(
        concat!(
            "  {} (Moved):\n",
            "    x: {:>7.2} mm -> {:>7.2} mm ({:+.2} mm)\n",
            "    y: {:>7.2} mm -> {:>7.2} mm ({:+.2} mm)\n"
        ),
        d.designator, cur[0], k[0], dx, cur[1], k[1], dy
    )
}
