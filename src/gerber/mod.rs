//! Native pure-Rust RS-274X Extended Gerber and Excellon NC drill generator.

pub mod apertures;
pub mod bundle;
pub mod coordinates;
pub mod excellon;
pub mod layers;
#[cfg(test)]
mod tests;
pub mod writer;

pub use apertures::{Aperture, ApertureDictionary};
pub use bundle::{create_gerber_zip, export_fab_bundle, generate_gerber_files, GerberFiles};
pub use coordinates::Coord;
pub use excellon::ExcellonWriter;
pub use layers::{GerberError, PcbModel};
pub use writer::{FilePolarity, GerberWriter};
