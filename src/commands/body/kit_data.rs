//! The reference kit data: board outline, holes, component footprints, and edge openings.

use std::collections::HashMap;

use serde::Serialize;

use super::edges::edge_contacts;
use super::spec::open_ring;
use super::{BodyError, DEFAULT_BOARD_THICKNESS};
use crate::gerber::layers::{PcbComponentElm, PcbPoint};
use crate::gerber::PcbModel;

#[derive(Debug, Serialize)]
pub(super) struct KitHole {
    pub x: f64,
    pub y: f64,
    pub diameter: f64,
    pub plated: bool,
}

/// A component footprint: `x`, `y` is its center, `w` and `h` its size.
#[derive(Debug, Serialize)]
pub(super) struct KitComponent {
    pub name: String,
    pub layer: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub touches_edge: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct KitOpening {
    pub component: String,
    pub edge: usize,
    pub side: &'static str,
    pub normal: [f64; 2],
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// Everything a designer needs to model a part around the board, in the generator's frame.
#[derive(Debug, Serialize)]
pub(super) struct Kit {
    pub board: Option<String>,
    pub units: &'static str,
    pub frame: &'static str,
    pub outline: Vec<[f64; 2]>,
    pub thickness: f64,
    pub holes: Vec<KitHole>,
    pub components: Vec<KitComponent>,
    pub openings: Vec<KitOpening>,
}

fn component_names(json: &str) -> HashMap<String, String> {
    let elements: Vec<serde_json::Value> = serde_json::from_str(json).unwrap_or_default();
    let text = |e: &serde_json::Value, key: &str| e[key].as_str().map(str::to_string);
    elements
        .iter()
        .filter(|e| e["type"] == "source_component")
        .filter_map(|e| {
            let id = text(e, "source_component_id").or_else(|| text(e, "component_id"))?;
            Some((id.clone(), text(e, "name").unwrap_or(id)))
        })
        .collect()
}

fn holes(model: &PcbModel) -> Vec<KitHole> {
    let plain = model.holes.iter().map(|h| {
        let d = h.hole_diameter.or(h.hole_width).unwrap_or(0.0);
        (h.x, h.y, d, false)
    });
    let plated = model.plated_holes.iter().map(|h| {
        let d = h.hole_diameter.or(h.hole_width).unwrap_or(0.0);
        (h.x, h.y, d, true)
    });
    plain
        .chain(plated)
        .map(|(x, y, diameter, plated)| KitHole {
            x,
            y,
            diameter,
            plated,
        })
        .collect()
}

fn side(normal: [f64; 2]) -> &'static str {
    let horizontal = normal[0].abs() >= normal[1].abs();
    match (horizontal, normal[0] >= 0.0, normal[1] >= 0.0) {
        (true, true, _) => "+x",
        (true, false, _) => "-x",
        (false, _, true) => "+y",
        (false, _, false) => "-y",
    }
}

fn label(component: &PcbComponentElm, names: &HashMap<String, String>) -> String {
    let id = component.source_component_id.clone();
    id.as_ref()
        .and_then(|id| names.get(id).cloned())
        .or(id)
        .or_else(|| component.pcb_component_id.clone())
        .unwrap_or_else(|| "component".into())
}

/// A component's kit entry and the wall openings it needs, or `None` without a footprint.
fn entry(
    c: &PcbComponentElm,
    outline: &[PcbPoint],
    names: &HashMap<String, String>,
) -> Option<(KitComponent, Vec<KitOpening>)> {
    let (center, w, h) = (c.center?, c.width?, c.height?);
    let name = label(c, names);
    let contacts = edge_contacts(c, outline);
    let openings: Vec<KitOpening> = contacts
        .iter()
        .map(|k| KitOpening {
            component: name.clone(),
            edge: k.edge,
            side: side(k.normal),
            normal: k.normal,
            x: center.x,
            y: center.y,
            w,
            h,
        })
        .collect();
    let component = KitComponent {
        layer: c.layer.clone().unwrap_or_else(|| "top".into()),
        touches_edge: !contacts.is_empty(),
        x: center.x,
        y: center.y,
        name,
        w,
        h,
    };
    Some((component, openings))
}

/// Builds the kit of the board in `json`; `board` is the manifest name written to the file.
pub(super) fn build_kit(json: &str, board: Option<String>) -> Result<Kit, BodyError> {
    let model = PcbModel::parse(json);
    let elm = model.boards.first().ok_or(BodyError::NoBoard)?;
    let outline = open_ring(elm.outline_points());
    if outline.len() < 3 {
        return Err(BodyError::NoBoard);
    }
    let names = component_names(json);
    let entries = model
        .components
        .iter()
        .filter_map(|c| entry(c, &outline, &names));
    let (components, openings): (Vec<_>, Vec<_>) = entries.unzip();
    Ok(Kit {
        board,
        units: "mm",
        frame: "pcb coordinates, Z-up; the enclosure generator uses the same X/Y",
        outline: outline.iter().map(|p| [p.x, p.y]).collect(),
        thickness: elm.thickness.unwrap_or(DEFAULT_BOARD_THICKNESS),
        holes: holes(&model),
        components,
        openings: openings.into_iter().flatten().collect(),
    })
}
