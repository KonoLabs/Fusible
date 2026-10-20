use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use super::spatial::SpatialRoutingIndex;

#[derive(Clone, Debug, PartialEq)]
pub enum RouteStepKind {
    Wire { layer: String, x: f64, y: f64 },
    Via { x: f64, y: f64 },
}

#[derive(Clone, Debug)]
pub struct RoutePath {
    pub steps: Vec<RouteStepKind>,
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
    pub via_cost: f64,
    pub turn_cost: f64,
    pub layers: Vec<String>,
}

impl Default for AStarConfig {
    fn default() -> Self {
        Self {
            grid_step: 0.25,
            trace_width: 0.2,
            clearance: 0.15,
            via_diameter: 0.6,
            via_cost: 15.0,
            turn_cost: 1.5,
            layers: vec!["top".to_string(), "bottom".to_string()],
        }
    }
}

pub fn find_path(
    start: [f64; 2],
    target: [f64; 2],
    net_id: &str,
    spatial: &SpatialRoutingIndex,
    config: &AStarConfig,
) -> Option<RoutePath> {
    let step = config.grid_step;
    let start_node = GridNode {
        gx: (start[0] / step).round() as i32,
        gy: (start[1] / step).round() as i32,
        layer_idx: 0,
    };
    let target_node = GridNode {
        gx: (target[0] / step).round() as i32,
        gy: (target[1] / step).round() as i32,
        layer_idx: 0,
    };

    let mut open_set = BinaryHeap::new();
    let mut g_score: HashMap<GridNode, f64> = HashMap::new();
    let mut came_from: HashMap<GridNode, GridNode> = HashMap::new();

    g_score.insert(start_node, 0.0);
    open_set.push(SearchState {
        cost: heuristic(start_node, target_node, step),
        node: start_node,
    });

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

    let max_expanded = 12000;
    let mut expansions = 0;

    while let Some(SearchState { cost: _, node }) = open_set.pop() {
        expansions += 1;
        if expansions > max_expanded {
            break;
        }

        if node.gx == target_node.gx && node.gy == target_node.gy {
            return Some(reconstruct_path(node, &came_from, step, &config.layers));
        }

        let current_g = match g_score.get(&node) {
            Some(&g) => g,
            None => continue,
        };

        let current_p = [node.gx as f64 * step, node.gy as f64 * step];
        let current_layer = &config.layers[node.layer_idx];

        for &(dx, dy) in &dirs {
            let next_node = GridNode {
                gx: node.gx + dx,
                gy: node.gy + dy,
                layer_idx: node.layer_idx,
            };
            let next_p = [next_node.gx as f64 * step, next_node.gy as f64 * step];

            let dist = if dx != 0 && dy != 0 {
                step * std::f64::consts::SQRT_2
            } else {
                step
            };

            let is_goal = next_node.gx == target_node.gx && next_node.gy == target_node.gy;
            let clear = is_goal
                || spatial.is_segment_clear(
                    current_layer,
                    current_p,
                    next_p,
                    config.trace_width,
                    config.clearance,
                    net_id,
                );

            if !clear {
                continue;
            }

            let tentative_g = current_g + dist;
            if tentative_g < *g_score.get(&next_node).unwrap_or(&f64::INFINITY) {
                g_score.insert(next_node, tentative_g);
                came_from.insert(next_node, node);
                let f_score = tentative_g + heuristic(next_node, target_node, step);
                open_set.push(SearchState {
                    cost: f_score,
                    node: next_node,
                });
            }
        }

        if config.layers.len() > 1 {
            let other_layer_idx = if node.layer_idx == 0 { 1 } else { 0 };
            let other_layer = &config.layers[other_layer_idx];
            let via_radius = config.via_diameter / 2.0;

            let via_clear = spatial.is_point_clear(
                current_layer,
                current_p,
                via_radius,
                config.clearance,
                net_id,
            ) && spatial.is_point_clear(
                other_layer,
                current_p,
                via_radius,
                config.clearance,
                net_id,
            );

            if via_clear {
                let next_node = GridNode {
                    gx: node.gx,
                    gy: node.gy,
                    layer_idx: other_layer_idx,
                };
                let tentative_g = current_g + config.via_cost;
                if tentative_g < *g_score.get(&next_node).unwrap_or(&f64::INFINITY) {
                    g_score.insert(next_node, tentative_g);
                    came_from.insert(next_node, node);
                    let f_score = tentative_g + heuristic(next_node, target_node, step);
                    open_set.push(SearchState {
                        cost: f_score,
                        node: next_node,
                    });
                }
            }
        }
    }

    None
}

fn heuristic(a: GridNode, b: GridNode, step: f64) -> f64 {
    let dx = (a.gx - b.gx).abs() as f64 * step;
    let dy = (a.gy - b.gy).abs() as f64 * step;
    let layer_penalty = if a.layer_idx != b.layer_idx {
        10.0
    } else {
        0.0
    };
    (dx.min(dy) * std::f64::consts::SQRT_2 + (dx - dy).abs()) + layer_penalty
}

fn reconstruct_path(
    end: GridNode,
    came_from: &HashMap<GridNode, GridNode>,
    step: f64,
    layers: &[String],
) -> RoutePath {
    let mut raw_nodes = vec![end];
    let mut curr = end;
    while let Some(&prev) = came_from.get(&curr) {
        raw_nodes.push(prev);
        curr = prev;
    }
    raw_nodes.reverse();

    let mut steps = Vec::new();
    for i in 0..raw_nodes.len() {
        let node = raw_nodes[i];
        let x = node.gx as f64 * step;
        let y = node.gy as f64 * step;

        if i > 0 && node.layer_idx != raw_nodes[i - 1].layer_idx {
            steps.push(RouteStepKind::Via { x, y });
        }

        steps.push(RouteStepKind::Wire {
            layer: layers[node.layer_idx].clone(),
            x,
            y,
        });
    }

    RoutePath {
        steps: compress_collinear(steps),
    }
}

fn compress_collinear(steps: Vec<RouteStepKind>) -> Vec<RouteStepKind> {
    if steps.len() <= 2 {
        return steps;
    }

    let mut compressed = Vec::new();
    compressed.push(steps[0].clone());

    for i in 1..steps.len() - 1 {
        let prev = &steps[i - 1];
        let curr = &steps[i];
        let next = &steps[i + 1];

        match (prev, curr, next) {
            (
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
            ) if l1 == l2 && l2 == l3 => {
                let dx1 = x2 - x1;
                let dy1 = y2 - y1;
                let dx2 = x3 - x2;
                let dy2 = y3 - y2;

                let cross = dx1 * dy2 - dy1 * dx2;
                if cross.abs() > 1e-6 {
                    compressed.push(curr.clone());
                }
            }
            _ => {
                compressed.push(curr.clone());
            }
        }
    }

    compressed.push(steps.last().unwrap().clone());
    compressed
}
