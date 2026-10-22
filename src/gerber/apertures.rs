//! Aperture definitions and deduplication dictionary for Gerber RS-274X.

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;

use crate::gerber::coordinates::Coord;
use crate::gerber::round_rect::write_round_rect;

/// Aperture geometry types supported in RS-274X standard template definitions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Aperture {
    Circle {
        diameter_nm: i64,
    },
    Rectangle {
        width_nm: i64,
        height_nm: i64,
    },
    Obround {
        width_nm: i64,
        height_nm: i64,
    },
    RoundRect {
        width_nm: i64,
        height_nm: i64,
        radius_nm: i64,
    },
}

impl Aperture {
    /// Builds a circle aperture, rejecting non-positive or non-finite diameters.
    pub fn circle(diameter_mm: f64) -> Result<Self, String> {
        Ok(Self::Circle {
            diameter_nm: size_nm(diameter_mm)?,
        })
    }

    /// Builds a rectangle aperture, rejecting non-positive or non-finite sizes.
    pub fn rectangle(width_mm: f64, height_mm: f64) -> Result<Self, String> {
        Ok(Self::Rectangle {
            width_nm: size_nm(width_mm)?,
            height_nm: size_nm(height_mm)?,
        })
    }

    /// Builds an obround aperture, rejecting non-positive or non-finite sizes.
    pub fn obround(width_mm: f64, height_mm: f64) -> Result<Self, String> {
        Ok(Self::Obround {
            width_nm: size_nm(width_mm)?,
            height_nm: size_nm(height_mm)?,
        })
    }

    /// Builds a rounded rectangle aperture drawn with a standard macro; the radius must leave a flat body.
    pub fn round_rect(width_mm: f64, height_mm: f64, radius_mm: f64) -> Result<Self, String> {
        let aperture = Self::RoundRect {
            width_nm: size_nm(width_mm)?,
            height_nm: size_nm(height_mm)?,
            radius_nm: size_nm(radius_mm)?,
        };
        match aperture {
            Self::RoundRect {
                width_nm,
                height_nm,
                radius_nm,
            } if 2 * radius_nm < width_nm.min(height_nm) => Ok(aperture),
            _ => Err(format!(
                "corner radius {radius_mm} mm must be smaller than half of the pad"
            )),
        }
    }
}

/// Rounds a millimeter size to whole nanometers and rejects anything that is not strictly positive.
fn size_nm(mm: f64) -> Result<i64, String> {
    if !mm.is_finite() {
        return Err(format!("aperture size {mm} is not a finite number"));
    }
    let nm = Coord::from_mm(mm).0;
    if nm <= 0 {
        return Err(format!("aperture size {mm} mm must be positive"));
    }
    Ok(nm)
}

/// An aperture dictionary that allocates contiguous D-codes starting at D10.
#[derive(Debug, Default)]
pub struct ApertureDictionary {
    map: HashMap<Aperture, u32>,
    ordered: Vec<(u32, Aperture)>,
    next_id: u32,
}

impl ApertureDictionary {
    /// Creates a new empty dictionary starting at D-code 10.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            ordered: Vec::new(),
            next_id: 10,
        }
    }

    /// Interns an aperture and returns its unique D-code.
    pub fn intern(&mut self, aperture: Aperture) -> u32 {
        if let Some(&id) = self.map.get(&aperture) {
            return id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.map.insert(aperture.clone(), id);
        self.ordered.push((id, aperture));
        id
    }

    /// Emits all aperture definition commands (%ADD...*%).
    pub fn write_definitions(&self, out: &mut String) {
        for (id, ap) in &self.ordered {
            match ap {
                Aperture::Circle { diameter_nm } => {
                    let d = *diameter_nm as f64 / 1_000_000.0;
                    let _ = writeln!(out, "%ADD{id}C,{:.6}*%", d);
                }
                Aperture::Rectangle {
                    width_nm,
                    height_nm,
                } => {
                    let w = *width_nm as f64 / 1_000_000.0;
                    let h = *height_nm as f64 / 1_000_000.0;
                    let _ = writeln!(out, "%ADD{id}R,{:.6}X{:.6}*%", w, h);
                }
                Aperture::Obround {
                    width_nm,
                    height_nm,
                } => {
                    let w = *width_nm as f64 / 1_000_000.0;
                    let h = *height_nm as f64 / 1_000_000.0;
                    let _ = writeln!(out, "%ADD{id}O,{:.6}X{:.6}*%", w, h);
                }
                Aperture::RoundRect {
                    width_nm,
                    height_nm,
                    radius_nm,
                } => write_round_rect(out, *id, [*width_nm, *height_nm, *radius_nm]),
            }
        }
    }
}
