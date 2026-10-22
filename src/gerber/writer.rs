//! RS-274X command stream writer.

use std::fmt::Write as FmtWrite;

use crate::gerber::apertures::ApertureDictionary;
use crate::gerber::coordinates::Coord;

/// Image polarity declared by the `.FilePolarity` attribute (Gerber spec 5.6.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilePolarity {
    Positive,
    Negative,
}

impl FilePolarity {
    fn as_str(self) -> &'static str {
        match self {
            Self::Positive => "Positive",
            Self::Negative => "Negative",
        }
    }
}

/// RS-274X command stream buffer writer.
pub struct GerberWriter {
    buffer: String,
    current_aperture: Option<u32>,
}

impl GerberWriter {
    /// Writes the file header: format, unit, file attributes, aperture dictionary, dark polarity and linear plot state.
    pub fn new(
        title: &str,
        file_function: &str,
        polarity: FilePolarity,
        apertures: &ApertureDictionary,
    ) -> Self {
        let mut buffer = String::with_capacity(8192);
        let _ = writeln!(buffer, "G04 =========================================*");
        let _ = writeln!(buffer, "G04 Fusible Pure-Rust Gerber Generator       *");
        let _ = writeln!(buffer, "G04 Layer: {title:<33} *");
        let _ = writeln!(buffer, "G04 =========================================*");
        let _ = writeln!(buffer, "%FSLAX46Y46*%");
        let _ = writeln!(buffer, "%MOMM*%");
        let _ = writeln!(
            buffer,
            "%TF.GenerationSoftware,Fusible,Fusible,{}*%",
            env!("CARGO_PKG_VERSION")
        );
        let _ = writeln!(buffer, "%TF.FileFunction,{file_function}*%");
        let _ = writeln!(buffer, "%TF.FilePolarity,{}*%", polarity.as_str());
        apertures.write_definitions(&mut buffer);
        let _ = writeln!(buffer, "%LPD*%");
        let _ = writeln!(buffer, "G01*");

        Self {
            buffer,
            current_aperture: None,
        }
    }

    /// Selects an aperture if not already selected.
    pub fn select_aperture(&mut self, id: u32) {
        if self.current_aperture != Some(id) {
            let _ = writeln!(self.buffer, "D{id}*");
            self.current_aperture = Some(id);
        }
    }

    /// Emits a travel move without exposure (D02).
    pub fn move_to(&mut self, x: Coord, y: Coord) {
        let _ = writeln!(
            self.buffer,
            "X{}Y{}D02*",
            x.format_gerber(),
            y.format_gerber()
        );
    }

    /// Emits a linear draw stroke with active aperture (D01).
    pub fn draw_to(&mut self, x: Coord, y: Coord) {
        let _ = writeln!(
            self.buffer,
            "X{}Y{}D01*",
            x.format_gerber(),
            y.format_gerber()
        );
    }

    /// Emits an aperture flash exposure (D03).
    pub fn flash(&mut self, x: Coord, y: Coord) {
        let _ = writeln!(
            self.buffer,
            "X{}Y{}D03*",
            x.format_gerber(),
            y.format_gerber()
        );
    }

    /// Emits a filled region (G36/G37) whose contour is closed back to its first vertex.
    pub fn region(&mut self, points: &[[Coord; 2]]) {
        let Some(&[fx, fy]) = points.first() else {
            return;
        };
        let _ = writeln!(self.buffer, "G36*");
        self.move_to(fx, fy);
        for &[x, y] in &points[1..] {
            self.draw_to(x, y);
        }
        self.draw_to(fx, fy);
        let _ = writeln!(self.buffer, "G37*");
    }

    /// Terminates the file with M02* and returns the serialized Gerber string.
    pub fn finish(mut self) -> String {
        let _ = writeln!(self.buffer, "M02*");
        self.buffer
    }
}
