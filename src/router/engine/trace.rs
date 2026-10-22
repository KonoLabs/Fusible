//! Inserting routed and pre-existing traces into the obstacle index.

use serde_json::Value;

use super::super::astar::{AStarConfig, RoutePath, RouteStepKind};
use super::super::spatial::{SpatialObstacle, SpatialRoutingIndex};
use super::json::{num, str_of};

/// Adds a freshly routed path's wires and vias as obstacles of `net_id`.
pub(super) fn insert_path(
    index: &mut SpatialRoutingIndex,
    net_id: &str,
    path: &RoutePath,
    config: &AStarConfig,
) {
    let net = Some(net_id.to_string());
    let wires = path.steps.windows(2).filter_map(|pair| match pair {
        [RouteStepKind::Wire { layer, x, y }, RouteStepKind::Wire {
            layer: l2,
            x: x2,
            y: y2,
        }] if layer == l2 => Some((layer, [*x, *y], [*x2, *y2])),
        _ => None,
    });
    for (layer, from, to) in wires {
        index.insert_obstacle(
            layer,
            SpatialObstacle::trace(net.clone(), from, to, config.trace_width),
        );
    }
    let vias = path.steps.iter().filter_map(|step| match step {
        RouteStepKind::Via { x, y, .. } => Some([*x, *y]),
        RouteStepKind::Wire { .. } => None,
    });
    for center in vias {
        for layer in &config.layers {
            index.insert_obstacle(
                layer,
                SpatialObstacle::via(net.clone(), center, config.via_diameter),
            );
        }
    }
}

/// Adds an existing `pcb_trace` route as obstacles of `net`.
pub(super) fn insert_existing_trace(
    index: &mut SpatialRoutingIndex,
    el: &Value,
    net: Option<String>,
    config: &AStarConfig,
) -> Result<(), String> {
    let route = el
        .get("route")
        .and_then(Value::as_array)
        .ok_or("missing route")?;
    let mut prev: Option<(&str, [f64; 2], f64)> = None;
    for pt in route {
        let p = num(pt, "x")
            .zip(num(pt, "y"))
            .map(|(x, y)| [x, y])
            .ok_or("route point missing x/y")?;
        if str_of(pt, "route_type") == Some("via") {
            let d = num(pt, "outer_diameter").unwrap_or(config.via_diameter);
            for layer in &config.layers {
                index.insert_obstacle(layer, SpatialObstacle::via(net.clone(), p, d));
            }
            prev = None;
            continue;
        }
        let layer = str_of(pt, "layer").ok_or("wire point missing layer")?;
        let width = num(pt, "width").ok_or("wire point missing width")?;
        if let Some((pl, pp, pw)) = prev.filter(|(pl, _, _)| *pl == layer) {
            index.insert_obstacle(pl, SpatialObstacle::trace(net.clone(), pp, p, pw));
        } else {
            index.insert_obstacle(layer, SpatialObstacle::via(net.clone(), p, width));
        }
        prev = Some((layer, p, width));
    }
    Ok(())
}
