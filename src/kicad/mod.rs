mod backup;
mod changes;
pub mod diff;
mod footprint;
mod live;
mod locate;
mod nets;
mod pads;
mod placements;
mod report;
pub mod sexpr;
pub mod sync;

#[cfg(test)]
mod backup_tests;
#[cfg(test)]
mod changes_tests;
#[cfg(test)]
mod drift_tests;
#[cfg(test)]
mod live_tests;
#[cfg(test)]
mod locate_tests;
#[cfg(test)]
mod sexpr_tests;
#[cfg(test)]
mod tests;

pub use backup::{backup_file, BACKUP_KEEP};
pub use diff::{
    compare_kicad_with_circuit, format_diff_report, DiffKind, DiffReport, PlacementDiff,
};
pub use live::{compare_board, sync_positions, Comparison};
pub use locate::board_file;
pub use pads::KicadPad;
pub use sexpr::{extract_footprints, parse_sexpr, KicadFootprint, Sexpr};
pub use sync::apply_kicad_sync;
