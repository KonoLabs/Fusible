//! Turning a solved grid search into route steps.

use std::collections::HashMap;

use super::node::GridNode;
use super::{RoutePath, RouteStepKind, Terminal};

/// Grid geometry and endpoints needed to turn grid nodes into route steps.
pub(super) struct PathFrame<'a> {
    pub step: f64,
    pub layers: &'a [String],
    pub start: &'a Terminal,
    pub target: &'a Terminal,
}

/// Walks `came_from` back from `end` and emits the route from the exact
/// start point to the exact target point, with vias at layer changes.
pub(super) fn reconstruct_path(
    end: GridNode,
    came_from: &HashMap<GridNode, GridNode>,
    frame: &PathFrame<'_>,
) -> RoutePath {
    let mut raw_nodes = vec![end];
    let mut curr = end;
    while let Some(&prev) = came_from.get(&curr) {
        raw_nodes.push(prev);
        curr = prev;
    }
    raw_nodes.reverse();

    let layers = frame.layers;
    let mut steps = vec![RouteStepKind::Wire {
        layer: layers[raw_nodes[0].layer_idx].clone(),
        x: frame.start.point[0],
        y: frame.start.point[1],
    }];
    push_node_steps(&mut steps, &raw_nodes, frame);
    steps.push(RouteStepKind::Wire {
        layer: layers[end.layer_idx].clone(),
        x: frame.target.point[0],
        y: frame.target.point[1],
    });

    RoutePath {
        steps: compress_collinear(steps),
    }
}

fn push_node_steps(steps: &mut Vec<RouteStepKind>, raw_nodes: &[GridNode], frame: &PathFrame<'_>) {
    let layers = frame.layers;
    for (i, node) in raw_nodes.iter().enumerate() {
        let x = node.gx as f64 * frame.step;
        let y = node.gy as f64 * frame.step;
        if i > 0 && node.layer_idx != raw_nodes[i - 1].layer_idx {
            steps.push(RouteStepKind::Via {
                x,
                y,
                from_layer: layers[raw_nodes[i - 1].layer_idx].clone(),
                to_layer: layers[node.layer_idx].clone(),
            });
        }
        steps.push(RouteStepKind::Wire {
            layer: layers[node.layer_idx].clone(),
            x,
            y,
        });
    }
}

fn compress_collinear(steps: Vec<RouteStepKind>) -> Vec<RouteStepKind> {
    if steps.len() <= 2 {
        return steps;
    }

    let mut compressed = vec![steps[0].clone()];
    for i in 1..steps.len() - 1 {
        if !is_redundant(&compressed[compressed.len() - 1], &steps[i], &steps[i + 1]) {
            compressed.push(steps[i].clone());
        }
    }
    compressed.push(steps[steps.len() - 1].clone());
    compressed
}

/// True when wire `b` lies on the straight, same-layer, forward run from `a` to `c`.
fn is_redundant(a: &RouteStepKind, b: &RouteStepKind, c: &RouteStepKind) -> bool {
    let (
        RouteStepKind::Wire {
            layer: l1,
            x: x1,
            y: y1,
        },
        RouteStepKind::Wire {
            layer: l2,
            x: x2,
            y: y2,
        },
        RouteStepKind::Wire {
            layer: l3,
            x: x3,
            y: y3,
        },
    ) = (a, b, c)
    else {
        return false;
    };
    let cross = (x2 - x1) * (y3 - y2) - (y2 - y1) * (x3 - x2);
    let backtracks = (x2 - x1) * (x3 - x2) + (y2 - y1) * (y3 - y2) < 0.0;
    l1 == l2 && l2 == l3 && cross.abs() <= 1e-9 && !backtracks
}
