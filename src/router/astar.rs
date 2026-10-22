use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use super::spatial::SpatialRoutingIndex;

const MAX_EXPANSIONS: usize = 400_000;

#[derive(Clone, Debug, PartialEq)]
pub enum RouteStepKind {
    Wire {
        layer: String,
        x: f64,
        y: f64,
    },
    Via {
        x: f64,
        y: f64,
        from_layer: String,
        to_layer: String,
    },
}

#[derive(Clone, Debug)]
pub struct RoutePath {
    pub steps: Vec<RouteStepKind>,
}

/// A connection endpoint: the exact pad center and the copper layers the pad exists on.
#[derive(Clone, Debug)]
pub struct Terminal {
    pub point: [f64; 2],
    pub layers: Vec<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct GridNode {
    gx: i32,
    gy: i32,
    layer_idx: usize,
}

#[derive(Copy, Clone)]
struct SearchState {
    cost: f64,
    node: GridNode,
}

impl PartialEq for SearchState {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for SearchState {}

impl Ord for SearchState {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for SearchState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct AStarConfig {
    pub grid_step: f64,
    pub trace_width: f64,
    pub clearance: f64,
    pub via_diameter: f64,
    pub via_hole_diameter: f64,
    pub via_cost: f64,
    pub layers: Vec<String>,
}

impl Default for AStarConfig {
    fn default() -> Self {
        Self {
            grid_step: 0.25,
            trace_width: 0.2,
            clearance: 0.15,
            via_diameter: 0.6,
            via_hole_diameter: 0.3,
            via_cost: 15.0,
            layers: vec!["top".to_string(), "bottom".to_string()],
        }
    }
}

/// Finds a clear path from `start` to `target`, leaving and entering only on layers the terminals exist on.
pub fn find_path(
    start: &Terminal,
    target: &Terminal,
    net_id: &str,
    spatial: &SpatialRoutingIndex,
    config: &AStarConfig,
) -> Option<RoutePath> {
    let step = config.grid_step;
    let snap = |p: [f64; 2]| ((p[0] / step).round() as i32, (p[1] / step).round() as i32);
    let at = |n: GridNode| [n.gx as f64 * step, n.gy as f64 * step];
    let layer_set = |t: &Terminal| -> Vec<usize> {
        (0..config.layers.len())
            .filter(|&i| t.layers.contains(&config.layers[i]))
            .collect()
    };
    let wire_clear = |layer: usize, a: [f64; 2], b: [f64; 2]| {
        spatial.is_segment_clear(
            &config.layers[layer],
            a,
            b,
            config.trace_width,
            config.clearance,
            net_id,
        )
    };

    let goal_layers = layer_set(target);
    let (sgx, sgy) = snap(start.point);
    let (tgx, tgy) = snap(target.point);
    let goal = GridNode {
        gx: tgx,
        gy: tgy,
        layer_idx: goal_layers.first().copied()?,
    };

    let mut open_set = BinaryHeap::new();
    let mut g_score: HashMap<GridNode, f64> = HashMap::new();
    let mut came_from: HashMap<GridNode, GridNode> = HashMap::new();
    let mut closed: HashSet<GridNode> = HashSet::new();

    for layer_idx in layer_set(start) {
        let node = GridNode {
            gx: sgx,
            gy: sgy,
            layer_idx,
        };
        if wire_clear(layer_idx, start.point, at(node)) {
            g_score.insert(node, 0.0);
            open_set.push(SearchState {
                cost: heuristic(node, goal, step),
                node,
            });
        }
    }

    let dirs = [
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
    ];

    while let Some(SearchState { node, .. }) = open_set.pop() {
        if !closed.insert(node) {
            continue;
        }
        if closed.len() > MAX_EXPANSIONS {
            break;
        }

        let current_p = at(node);
        if node.gx == tgx
            && node.gy == tgy
            && goal_layers.contains(&node.layer_idx)
            && wire_clear(node.layer_idx, current_p, target.point)
        {
            return Some(reconstruct_path(
                node,
                &came_from,
                step,
                &config.layers,
                start,
                target,
            ));
        }

        let current_g = g_score[&node];
        let mut relax = |next: GridNode, cost: f64| {
            let tentative_g = current_g + cost;
            if tentative_g < *g_score.get(&next).unwrap_or(&f64::INFINITY) {
                g_score.insert(next, tentative_g);
                came_from.insert(next, node);
                open_set.push(SearchState {
                    cost: tentative_g + heuristic(next, goal, step),
                    node: next,
                });
            }
        };

        for &(dx, dy) in &dirs {
            let next = GridNode {
                gx: node.gx + dx,
                gy: node.gy + dy,
                layer_idx: node.layer_idx,
            };
            if closed.contains(&next) || !wire_clear(node.layer_idx, current_p, at(next)) {
                continue;
            }
            let dist = if dx != 0 && dy != 0 {
                step * std::f64::consts::SQRT_2
            } else {
                step
            };
            relax(next, dist);
        }

        let via_clear = || {
            config.layers.iter().all(|layer| {
                spatial.is_segment_clear(
                    layer,
                    current_p,
                    current_p,
                    config.via_diameter,
                    config.clearance,
                    net_id,
                )
            })
        };
        let other_layers: Vec<usize> = (0..config.layers.len())
            .filter(|&i| i != node.layer_idx)
            .collect();
        if !other_layers.is_empty() && via_clear() {
            for layer_idx in other_layers {
                relax(GridNode { layer_idx, ..node }, config.via_cost);
            }
        }
    }

    None
}

fn heuristic(a: GridNode, b: GridNode, step: f64) -> f64 {
    let dx = (a.gx - b.gx).abs() as f64 * step;
    let dy = (a.gy - b.gy).abs() as f64 * step;
    dx.min(dy) * std::f64::consts::SQRT_2 + (dx - dy).abs()
}

fn reconstruct_path(
    end: GridNode,
    came_from: &HashMap<GridNode, GridNode>,
    step: f64,
    layers: &[String],
    start: &Terminal,
    target: &Terminal,
) -> RoutePath {
    let mut raw_nodes = vec![end];
    let mut curr = end;
    while let Some(&prev) = came_from.get(&curr) {
        raw_nodes.push(prev);
        curr = prev;
    }
    raw_nodes.reverse();

    let mut steps = vec![RouteStepKind::Wire {
        layer: layers[raw_nodes[0].layer_idx].clone(),
        x: start.point[0],
        y: start.point[1],
    }];
    for (i, node) in raw_nodes.iter().enumerate() {
        let x = node.gx as f64 * step;
        let y = node.gy as f64 * step;
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
    steps.push(RouteStepKind::Wire {
        layer: layers[end.layer_idx].clone(),
        x: target.point[0],
        y: target.point[1],
    });

    RoutePath {
        steps: compress_collinear(steps),
    }
}

fn compress_collinear(steps: Vec<RouteStepKind>) -> Vec<RouteStepKind> {
    if steps.len() <= 2 {
        return steps;
    }

    let mut compressed = vec![steps[0].clone()];
    for i in 1..steps.len() - 1 {
        if let (
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
        ) = (&compressed[compressed.len() - 1], &steps[i], &steps[i + 1])
        {
            let cross = (x2 - x1) * (y3 - y2) - (y2 - y1) * (x3 - x2);
            let backtracks = (x2 - x1) * (x3 - x2) + (y2 - y1) * (y3 - y2) < 0.0;
            if l1 == l2 && l2 == l3 && cross.abs() <= 1e-9 && !backtracks {
                continue;
            }
        }
        compressed.push(steps[i].clone());
    }
    compressed.push(steps[steps.len() - 1].clone());
    compressed
}
