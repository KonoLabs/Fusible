pub mod diff;
pub mod sexpr;
pub mod sync;

#[cfg(test)]
mod tests;

pub use diff::{
    compare_kicad_with_circuit, format_diff_report, DiffKind, DiffReport, PlacementDiff,
};
pub use sexpr::{extract_footprints, parse_sexpr, KicadFootprint, Sexpr};
pub use sync::apply_kicad_sync;
