//! Copper layer stack naming and pad side resolution.

use super::elements::PcbSmtPadElm;

/// A copper layer in stack order: `Top` is L1, `Inner(n)` is L(n+1), `Bottom` is the last layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CopperLayer {
    Top,
    Inner(u8),
    Bottom,
}

impl CopperLayer {
    /// Parses a tscircuit layer name: `top`, `bottom`, or `innerN` with N >= 1.
    pub fn parse(name: Option<&str>) -> Result<Self, String> {
        match name {
            Some("top") => Ok(Self::Top),
            Some("bottom") => Ok(Self::Bottom),
            Some(other) => other
                .strip_prefix("inner")
                .and_then(|n| n.parse::<u8>().ok())
                .filter(|n| *n >= 1)
                .map(Self::Inner)
                .ok_or_else(|| format!("unsupported copper layer '{other}'")),
            None => Err("missing layer".to_string()),
        }
    }

    /// Returns the KiCad-style file name, e.g. `F_Cu.gbr`, `In1_Cu.gbr`, `B_Cu.gbr`.
    pub fn file_name(self) -> String {
        match self {
            Self::Top => "F_Cu.gbr".to_string(),
            Self::Inner(n) => format!("In{n}_Cu.gbr"),
            Self::Bottom => "B_Cu.gbr".to_string(),
        }
    }

    /// Returns the Gerber title and `FileFunction` attribute for this layer in a `layer_count` stack.
    pub(super) fn title_and_function(self, layer_count: u8) -> (String, String) {
        match self {
            Self::Top => ("Top Copper (F_Cu)".to_string(), "Copper,L1,Top".to_string()),
            Self::Inner(n) => (
                format!("Inner Copper {n} (In{n}_Cu)"),
                format!("Copper,L{},Inr", n + 1),
            ),
            Self::Bottom => (
                "Bottom Copper (B_Cu)".to_string(),
                format!("Copper,L{layer_count},Bot"),
            ),
        }
    }
}

/// Returns true when an outer-layer pad sits on the top side.
pub(super) fn pad_is_top(pad: &PcbSmtPadElm) -> Result<bool, String> {
    match pad.layer.as_deref() {
        Some("top") => Ok(true),
        Some("bottom") => Ok(false),
        Some(other) => Err(format!("unsupported copper layer '{other}'")),
        None => Err("missing layer".to_string()),
    }
}

/// Returns true when the pad sits on the requested side and is not covered by solder mask.
pub(super) fn pad_exposed(pad: &PcbSmtPadElm, is_top: bool) -> Result<bool, String> {
    Ok(pad_is_top(pad)? == is_top && pad.is_covered_with_solder_mask != Some(true))
}
