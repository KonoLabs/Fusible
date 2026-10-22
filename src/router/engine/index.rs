//! Building the obstacle index and per-net pad terminals from Circuit JSON.

use std::collections::BTreeMap;

use serde_json::Value;

use super::super::astar::{AStarConfig, Terminal};
use super::super::spatial::{SpatialObstacle, SpatialRoutingIndex};
use super::connectivity::Connectivity;
use super::json::{num, pad_half_extent, str_of, type_of};
use super::trace::insert_existing_trace;
use crate::gerber::layers::PcbBoardElm;

pub(super) type NetTerminals = BTreeMap<String, Vec<Terminal>>;

const DIAMETER_KEYS: [&str; 4] = [
    "outer_diameter",
    "hole_diameter",
    "hole_width",
    "hole_height",
];

/// Accumulates obstacles and pad terminals while scanning elements.
struct IndexBuilder<'a> {
    index: SpatialRoutingIndex,
    nets: NetTerminals,
    config: &'a AStarConfig,
    trace_net: BTreeMap<String, String>,
}

/// Builds the obstacle index (board outline, pads, plated and bare holes,
/// vias, existing traces) and groups pad terminals by net.
pub(super) fn build_index(
    elements: &[Value],
    config: &AStarConfig,
) -> Result<(SpatialRoutingIndex, NetTerminals), String> {
    let mut conn = Connectivity::from_elements(elements);
    let port_to_source: BTreeMap<&str, &str> = elements
        .iter()
        .filter(|el| type_of(el) == "pcb_port")
        .filter_map(|el| Some((str_of(el, "pcb_port_id")?, str_of(el, "source_port_id")?)))
        .collect();
    let trace_net = trace_nets(elements, &mut conn);
    let mut builder = IndexBuilder {
        index: SpatialRoutingIndex::new(),
        nets: BTreeMap::new(),
        config,
        trace_net,
    };
    builder.index.set_boundary(board_outline(elements)?);
    for (i, el) in elements.iter().enumerate() {
        let typ = type_of(el);
        let net = str_of(el, "pcb_port_id")
            .and_then(|p| port_to_source.get(p))
            .and_then(|sp| conn.net_of(sp));
        builder
            .add(el, typ, net)
            .map_err(|what| format!("{typ} at JSON index {i}: {what}"))?;
    }
    Ok((builder.index, builder.nets))
}

/// Maps each `source_trace` id to the net of its first connected member.
fn trace_nets(elements: &[Value], conn: &mut Connectivity) -> BTreeMap<String, String> {
    elements
        .iter()
        .filter(|el| type_of(el) == "source_trace")
        .filter_map(|el| {
            let id = str_of(el, "source_trace_id")?;
            let member = ["connected_source_port_ids", "connected_source_net_ids"]
                .iter()
                .filter_map(|key| el.get(*key).and_then(Value::as_array))
                .flatten()
                .find_map(Value::as_str)?;
            Some((id.to_string(), member.to_string()))
        })
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .filter_map(|(id, member)| Some((id, conn.net_of(&member)?)))
        .collect()
}

/// The board outline polygon; a board without an outline or a full size is an error.
fn board_outline(elements: &[Value]) -> Result<Vec<[f64; 2]>, String> {
    let board = elements
        .iter()
        .find(|el| type_of(el) == "pcb_board")
        .ok_or("Circuit JSON has no pcb_board to bound routing")?;
    let board: PcbBoardElm =
        serde_json::from_value(board.clone()).map_err(|e| format!("malformed pcb_board: {e}"))?;
    if board.outline.is_none() && (board.width.is_none() || board.height.is_none()) {
        return Err("pcb_board has neither an outline nor width and height".to_string());
    }
    Ok(board.outline_points().iter().map(|p| [p.x, p.y]).collect())
}

impl IndexBuilder<'_> {
    fn add(&mut self, el: &Value, typ: &str, net: Option<String>) -> Result<(), String> {
        let at = || num(el, "x").zip(num(el, "y")).ok_or("missing x/y");
        match typ {
            "pcb_smtpad" => self.add_smtpad(el, at()?, net),
            "pcb_plated_hole" => self.add_plated_hole(el, at()?, net),
            "pcb_hole" | "pcb_via" => self.add_hole(el, at()?),
            "pcb_trace" => {
                let net = str_of(el, "source_trace_id")
                    .and_then(|t| self.trace_net.get(t).cloned())
                    .or_else(|| str_of(el, "net_id").map(str::to_owned));
                insert_existing_trace(&mut self.index, el, net, self.config)
            }
            _ => Ok(()),
        }
    }

    fn add_smtpad(
        &mut self,
        el: &Value,
        (x, y): (f64, f64),
        net: Option<String>,
    ) -> Result<(), String> {
        let layer = str_of(el, "layer").ok_or("missing layer")?;
        let [hw, hh] = pad_half_extent(el).ok_or("missing pad size")?;
        self.index.insert_obstacle(
            layer,
            SpatialObstacle::pad(net.clone(), [x - hw, y - hh], [x + hw, y + hh]),
        );
        if let Some(net) = net {
            self.nets.entry(net).or_default().push(Terminal {
                point: [x, y],
                layers: vec![layer.to_string()],
            });
        }
        Ok(())
    }

    fn add_plated_hole(
        &mut self,
        el: &Value,
        (x, y): (f64, f64),
        net: Option<String>,
    ) -> Result<(), String> {
        let [hw, hh] = pad_half_extent(el).ok_or("missing pad size")?;
        for layer in &self.config.layers {
            self.index.insert_obstacle(
                layer,
                SpatialObstacle::pad(net.clone(), [x - hw, y - hh], [x + hw, y + hh]),
            );
        }
        if let Some(net) = net {
            self.nets.entry(net).or_default().push(Terminal {
                point: [x, y],
                layers: self.config.layers.clone(),
            });
        }
        Ok(())
    }

    fn add_hole(&mut self, el: &Value, (x, y): (f64, f64)) -> Result<(), String> {
        let d = DIAMETER_KEYS
            .iter()
            .filter_map(|k| num(el, k))
            .fold(None, |m: Option<f64>, v| Some(m.map_or(v, |m| m.max(v))))
            .ok_or("missing diameter")?;
        for layer in &self.config.layers {
            self.index
                .insert_obstacle(layer, SpatialObstacle::via(None, [x, y], d));
        }
        Ok(())
    }
}
