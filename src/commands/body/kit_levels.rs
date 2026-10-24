//! Component heights and keep-out volumes of the reference kit.

use std::collections::HashMap;

use serde::Serialize;

use super::kit_data::KitComponent;
use super::BodyError;
use crate::gerber::layers::PcbComponentElm;

/// The volume a component occupies: its footprint box from `z0` to `z1` (cad Z, mm).
#[derive(Debug, Serialize)]
pub(super) struct KitKeepOut {
    pub component: String,
    pub layer: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub z0: f64,
    pub z1: f64,
}

/// Bottom and height of each component's 3D model, keyed by `pcb_component_id`.
pub(super) fn component_heights(json: &str) -> Result<HashMap<String, (f64, f64)>, BodyError> {
    let elements: Vec<serde_json::Value> =
        serde_json::from_str(json).map_err(|e| BodyError::InvalidCircuit(e.to_string()))?;
    Ok(elements
        .iter()
        .filter(|e| e["type"] == "cad_component")
        .filter_map(|e| {
            let id = e["pcb_component_id"].as_str()?.to_string();
            let height = e["size"]["z"]
                .as_f64()
                .filter(|h| h.is_finite() && *h > 0.0)?;
            let center = e["position"]["z"].as_f64().filter(|z| z.is_finite())?;
            Some((id, (center - height / 2.0, height)))
        })
        .collect())
}

/// Bottom and height of `component`'s 3D model, when the circuit has one.
pub(super) fn level_of(
    component: &PcbComponentElm,
    heights: &HashMap<String, (f64, f64)>,
) -> Option<(f64, f64)> {
    let id = component.pcb_component_id.as_ref()?;
    heights.get(id).copied()
}

pub(super) fn keep_out(components: &[KitComponent]) -> Vec<KitKeepOut> {
    components
        .iter()
        .filter_map(|c| {
            let (z, height) = (c.z?, c.height?);
            Some(KitKeepOut {
                component: c.name.clone(),
                layer: c.layer.clone(),
                x: c.x,
                y: c.y,
                w: c.w,
                h: c.h,
                z0: z,
                z1: z + height,
            })
        })
        .collect()
}

pub(super) fn warnings(skipped_holes: usize, unmeasured: usize) -> Vec<String> {
    let holes = (skipped_holes > 0)
        .then(|| format!("{skipped_holes} hole(s) have no diameter and are not in the kit"));
    let heights = (unmeasured > 0).then(|| {
        format!("{unmeasured} component(s) have no 3D model height and no keep_out volume")
    });
    holes.into_iter().chain(heights).collect()
}
