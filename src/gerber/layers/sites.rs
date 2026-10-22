//! Via sites, trace segments and the copper layer stack they imply.

use std::collections::BTreeSet;

use super::copper::CopperLayer;
use super::elements::{PcbModel, PcbTracePoint, PcbViaElm};
use super::math::{coord, required};
use super::shapes::trace_segment;
use super::{invalid, GerberError};

#[derive(Debug, Clone, Copy)]
pub(super) struct ViaSite {
    pub(super) at: [f64; 2],
    pub(super) outer: f64,
    pub(super) hole: f64,
    pub(super) tented: bool,
}

#[derive(Debug)]
pub(super) struct TraceSegment {
    pub(super) layer: CopperLayer,
    pub(super) width: f64,
    pub(super) from: [f64; 2],
    pub(super) to: [f64; 2],
}

impl PcbModel {
    /// Returns every copper layer of the stack, from the board's `num_layers` and the inner layers traces use.
    pub fn copper_layers(&self) -> Result<Vec<CopperLayer>, GerberError> {
        let declared = self
            .boards
            .iter()
            .filter_map(|b| b.num_layers)
            .max()
            .unwrap_or(2)
            .max(2);
        for (i, seg) in self.trace_segments()?.into_iter().enumerate() {
            if let CopperLayer::Inner(n) = seg.layer {
                if n > declared - 2 {
                    return Err(invalid("pcb_trace segment", i)(format!(
                        "uses inner{n} but the board declares {declared} layers"
                    )));
                }
            }
        }
        let mut layers = vec![CopperLayer::Top];
        layers.extend((1..=declared - 2).map(CopperLayer::Inner));
        layers.push(CopperLayer::Bottom);
        Ok(layers)
    }

    /// Returns every distinct via: explicit `pcb_via` elements first, then trace vias at new positions.
    pub(super) fn via_sites(&self) -> Result<Vec<ViaSite>, GerberError> {
        let mut seen = BTreeSet::new();
        let mut sites = Vec::new();
        for (i, via) in self.vias.iter().enumerate() {
            let site = pcb_via_site(via).map_err(invalid("pcb_via", i))?;
            seen.insert(coord(site.at));
            sites.push(site);
        }
        for (i, trace) in self.traces.iter().enumerate() {
            for pt in &trace.route {
                if pt.route_type.as_deref() != Some("via") || !seen.insert(coord([pt.x, pt.y])) {
                    continue;
                }
                sites.push(route_via_site(pt).map_err(invalid("pcb_trace", i))?);
            }
        }
        Ok(sites)
    }

    /// Returns every straight segment of every trace route.
    pub(super) fn trace_segments(&self) -> Result<Vec<TraceSegment>, GerberError> {
        let mut segments = Vec::new();
        for (i, trace) in self.traces.iter().enumerate() {
            for win in trace.route.windows(2) {
                segments.push(trace_segment(&win[0], &win[1]).map_err(invalid("pcb_trace", i))?);
            }
        }
        Ok(segments)
    }
}

/// Returns the via site of an explicit `pcb_via` element.
fn pcb_via_site(via: &PcbViaElm) -> Result<ViaSite, String> {
    Ok(ViaSite {
        at: [via.x, via.y],
        outer: required(via.outer_diameter, "outer_diameter")?,
        hole: required(via.hole_diameter, "hole_diameter")?,
        tented: via.is_tented != Some(false),
    })
}

/// Returns the tented via site of a trace via route point with no matching `pcb_via`.
fn route_via_site(pt: &PcbTracePoint) -> Result<ViaSite, String> {
    Ok(ViaSite {
        at: [pt.x, pt.y],
        outer: required(
            pt.outer_diameter,
            "via outer_diameter (no pcb_via at this position)",
        )?,
        hole: required(
            pt.hole_diameter,
            "via hole_diameter (no pcb_via at this position)",
        )?,
        tented: true,
    })
}
