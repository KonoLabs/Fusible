//! Fixed-point nanometer coordinates for Gerber RS-274X and Excellon drill.

/// Fixed-point coordinate stored in nanometers (1 nm = 1e-6 mm).
/// Eliminates IEEE-754 floating point rounding drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Coord(pub i64);

impl Coord {
    pub const ZERO: Self = Self(0);

    /// Converts millimeters into nanometer fixed-point coordinate.
    pub fn from_mm(mm: f64) -> Self {
        Self((mm * 1_000_000.0).round() as i64)
    }

    /// Converts nanometer coordinate back to millimeters.
    pub fn to_mm(self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }

    /// Formats coordinate for RS-274X %FSLAX46Y46*% (leading zero omission).
    /// e.g. 1.25 mm -> "1250000", -3.4 mm -> "-3400000", 0.0 mm -> "0"
    pub fn format_gerber(self) -> String {
        self.0.to_string()
    }

    /// Formats coordinate for Excellon drill with explicit 4 decimal places (e.g. "12.5000").
    pub fn format_drill(self) -> String {
        format!("{:.4}", self.to_mm())
    }
}
