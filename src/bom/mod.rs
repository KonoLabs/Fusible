//! Bill of materials: parts detected in the Circuit JSON, printed bodies, and manual parts.
//!
//! Prices and picks live in `bom.yml` next to `fusible.yml`. Fusible detects the electronic
//! parts, and the file only holds what the circuit cannot know.

mod detect;
mod error;
mod estimate;
mod file;
mod project;
mod render;
mod report;
mod stl;
#[cfg(test)]
mod stl_tests;
#[cfg(test)]
mod tests;
mod value;

pub use detect::{detect, Part};
pub use error::BomError;
pub use estimate::Body;
pub use file::{bom_path, BomFile, PartEntry, PrintEntry, PrintSettings};
pub use project::bodies_of;
pub use render::{csv, table};
pub use report::{build, Kind, Line, Report, Totals};
pub use stl::stl_grams;
