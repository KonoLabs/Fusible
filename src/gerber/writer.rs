//! RS-274X command stream writer.

use std::fmt::Write as FmtWrite;

use crate::gerber::apertures::ApertureDictionary;
use crate::gerber::coordinates::Coord;

/// RS-274X command stream buffer writer.
pub struct GerberWriter {
    buffer: String,
    current_aperture: Option<u32>,
}

impl GerberWriter {
    /// Initializes a new Gerber writer with standard header comments.
    pub fn new(title: &str, file_function: &str, polarity_dark: bool) -> Self {
        let mut buffer = String::with_capacity(8192);
        let _ = writeln!(buffer, "G04 =========================================*");
        let _ = writeln!(buffer, "G04 Fusible Pure-Rust Gerber Generator       *");
        let _ = writeln!(buffer, "G04 Layer: {title:<33} *");
        let _ = writeln!(buffer, "G04 =========================================*");
        let _ = writeln!(buffer, "%FSLAX46Y46*%");
        let _ = writeln!(buffer, "%MOMM*%");
        let _ = writeln!(
            buffer,
            "{}",
            if polarity_dark { "%LPD*%" } else { "%LPC*%" }
        );
        let _ = writeln!(buffer, "%TF.GenerationSoftware,Fusible,0.3.1*%");
        let _ = writeln!(buffer, "%TF.FileFunction,{file_function}*%");
        let _ = writeln!(buffer, "%TF.FilePolarity,Positive*%");

        Self {
            buffer,
            current_aperture: None,
        }
    }

    /// Appends the aperture dictionary definitions to the header.
    pub fn write_apertures(&mut self, apertures: &ApertureDictionary) {
        apertures.write_definitions(&mut self.buffer);
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

    /// Terminates the file with M02* and returns the serialized Gerber string.
    pub fn finish(mut self) -> String {
        let _ = writeln!(self.buffer, "M02*");
        self.buffer
    }
}
