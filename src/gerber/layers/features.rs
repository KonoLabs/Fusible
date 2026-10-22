//! Feature collectors turning PCB elements into Gerber flashes, strokes and regions.

use super::copper::CopperLayer;
use super::elements::{PcbModel, PcbPlatedHoleElm, PcbSmtPadElm};
use super::geometry::{Feature, PadShape};
use super::holes::plated_hole_geometry;
use super::invalid;
use super::shapes::smtpad_shape;
use super::GerberError;
use crate::gerber::apertures::Aperture;

const DEFAULT_SILK_STROKE_MM: f64 = 0.15;

impl PcbModel {
    /// Returns the SMT pad features, keeping each pad for which `margin_of` yields a mask margin.
    pub(super) fn smtpad_features(
        &self,
        margin_of: impl Fn(&PcbSmtPadElm) -> Result<Option<f64>, String>,
    ) -> Result<Vec<Feature>, GerberError> {
        let mut features = Vec::new();
        for (i, pad) in self.smt_pads.iter().enumerate() {
            let feature = margin_of(pad)
                .and_then(|margin| {
                    margin
                        .map(|m| smtpad_shape(pad)?.feature([pad.x, pad.y], m))
                        .transpose()
                })
                .map_err(invalid("pcb_smtpad", i))?;
            features.extend(feature);
        }
        Ok(features)
    }

    /// Returns the plated hole ring features, keeping each hole for which `margin_of` yields a margin.
    pub(super) fn plated_ring_features(
        &self,
        margin_of: impl Fn(&PcbPlatedHoleElm) -> Result<Option<f64>, String>,
    ) -> Result<Vec<Feature>, GerberError> {
        let mut features = Vec::new();
        for (i, hole) in self.plated_holes.iter().enumerate() {
            let feature = margin_of(hole)
                .and_then(|margin| {
                    margin
                        .map(|m| plated_hole_geometry(hole)?.0.feature([hole.x, hole.y], m))
                        .transpose()
                })
                .map_err(invalid("pcb_plated_hole", i))?;
            features.extend(feature);
        }
        Ok(features)
    }

    /// Returns the via pad features grown by `margin`, skipping tented vias unless `include_tented`.
    pub(super) fn via_pad_features(
        &self,
        margin: f64,
        include_tented: bool,
    ) -> Result<Vec<Feature>, GerberError> {
        let mut features = Vec::new();
        for (i, via) in self.via_sites()?.into_iter().enumerate() {
            if include_tented || !via.tented {
                features.push(
                    PadShape::Circle { d: via.outer }
                        .feature(via.at, margin)
                        .map_err(invalid("via", i))?,
                );
            }
        }
        Ok(features)
    }

    /// Returns the trace strokes routed on `layer`.
    pub(super) fn trace_features(&self, layer: CopperLayer) -> Result<Vec<Feature>, GerberError> {
        let mut features = Vec::new();
        for (i, seg) in self.trace_segments()?.into_iter().enumerate() {
            if seg.layer == layer {
                let ap = Aperture::circle(seg.width).map_err(invalid("pcb_trace segment", i))?;
                features.push(Feature::Stroke(ap, seg.from, seg.to));
            }
        }
        Ok(features)
    }

    /// Returns the silkscreen path strokes on one side; untagged paths belong to the top.
    pub(super) fn silkscreen_features(&self, is_top: bool) -> Result<Vec<Feature>, GerberError> {
        let layer_name = if is_top { "top" } else { "bottom" };
        let mut features = Vec::new();
        for (i, path) in self.silkscreen_paths.iter().enumerate() {
            let on_side =
                path.layer.as_deref() == Some(layer_name) || (is_top && path.layer.is_none());
            let width = path.stroke_width.unwrap_or(DEFAULT_SILK_STROKE_MM);
            if !on_side || width == 0.0 {
                continue;
            }
            let ap = Aperture::circle(width).map_err(invalid("pcb_silkscreen_path", i))?;
            features.extend(path.route.windows(2).map(|win| {
                Feature::Stroke(ap.clone(), [win[0].x, win[0].y], [win[1].x, win[1].y])
            }));
        }
        Ok(features)
    }
}
