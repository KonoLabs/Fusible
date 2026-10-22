//! Copper, solder mask, paste and silkscreen Gerber layer builders.

use super::copper::{pad_exposed, CopperLayer};
use super::elements::PcbModel;
use super::geometry::render;
use super::math::finite_or;
use super::GerberError;
use crate::gerber::writer::FilePolarity;

const DEFAULT_MASK_MARGIN_MM: f64 = 0.05;

impl PcbModel {
    /// Builds a copper layer: SMT pads, plated hole rings, via pads and trace strokes.
    pub fn build_copper_layer(&self, layer: CopperLayer) -> Result<String, GerberError> {
        let layer_count = self.copper_layers()?.len() as u8;
        let mut features = self.smtpad_features(|pad| {
            Ok((CopperLayer::parse(pad.layer.as_deref())? == layer).then_some(0.0))
        })?;
        features.extend(self.plated_ring_features(|_| Ok(Some(0.0)))?);
        features.extend(self.via_pad_features(0.0, true)?);
        features.extend(self.trace_features(layer)?);

        let (title, func) = layer.title_and_function(layer_count);
        Ok(render(&title, &func, FilePolarity::Positive, &features))
    }

    /// Builds a negative-polarity solder mask layer whose image is the set of mask openings.
    pub fn build_mask_layer(&self, is_top: bool) -> Result<String, GerberError> {
        let mut features = self.smtpad_features(|pad| {
            if !pad_exposed(pad, is_top)? {
                return Ok(None);
            }
            mask_margin(pad.soldermask_margin).map(Some)
        })?;
        features.extend(self.plated_ring_features(|hole| {
            if hole.is_covered_with_solder_mask == Some(true) {
                return Ok(None);
            }
            mask_margin(hole.soldermask_margin).map(Some)
        })?);
        features.extend(self.via_pad_features(DEFAULT_MASK_MARGIN_MM, false)?);

        let (title, func) = if is_top {
            ("Top Solder Mask (F_Mask)", "Soldermask,Top")
        } else {
            ("Bottom Solder Mask (B_Mask)", "Soldermask,Bot")
        };
        Ok(render(title, func, FilePolarity::Negative, &features))
    }

    /// Builds a solder paste layer from the exposed SMT pads on one side.
    pub fn build_paste_layer(&self, is_top: bool) -> Result<String, GerberError> {
        let features = self.smtpad_features(|pad| Ok(pad_exposed(pad, is_top)?.then_some(0.0)))?;

        let (title, func) = if is_top {
            ("Top Solder Paste (F_Paste)", "Paste,Top")
        } else {
            ("Bottom Solder Paste (B_Paste)", "Paste,Bot")
        };
        Ok(render(title, func, FilePolarity::Positive, &features))
    }

    /// Builds a silkscreen layer from silkscreen paths.
    pub fn build_silkscreen_layer(&self, is_top: bool) -> Result<String, GerberError> {
        let features = self.silkscreen_features(is_top)?;

        let (title, func) = if is_top {
            ("Top Silkscreen (F_Silk)", "Legend,Top")
        } else {
            ("Bottom Silkscreen (B_Silk)", "Legend,Bot")
        };
        Ok(render(title, func, FilePolarity::Positive, &features))
    }
}

/// Returns the solder mask expansion, defaulting when the element sets none.
fn mask_margin(margin: Option<f64>) -> Result<f64, String> {
    finite_or(margin, DEFAULT_MASK_MARGIN_MM, "soldermask_margin")
}
