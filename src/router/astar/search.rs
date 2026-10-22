//! The A* grid search state and its expansion steps.

use std::collections::{BinaryHeap, HashMap, HashSet};

use super::node::{heuristic, layer_indices, snap, GridNode, SearchState};
use super::path::{reconstruct_path, PathFrame};
use super::{AStarConfig, RoutePath, Terminal};
use crate::router::spatial::{Probe, SpatialRoutingIndex};

const MAX_EXPANSIONS: usize = 400_000;

const DIRS: [(i32, i32); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

/// One A* search from a start terminal to a target terminal.
pub(super) struct Search<'a> {
    spatial: &'a SpatialRoutingIndex,
    config: &'a AStarConfig,
    start: &'a Terminal,
    target: &'a Terminal,
    wire: Probe<'a>,
    via: Probe<'a>,
    goal: GridNode,
    goal_layers: Vec<usize>,
    open_set: BinaryHeap<SearchState>,
    g_score: HashMap<GridNode, f64>,
    came_from: HashMap<GridNode, GridNode>,
    closed: HashSet<GridNode>,
}

impl<'a> Search<'a> {
    /// Prepares a search, or `None` when the target exists on no routing layer.
    pub(super) fn new(
        start: &'a Terminal,
        target: &'a Terminal,
        net_id: &'a str,
        spatial: &'a SpatialRoutingIndex,
        config: &'a AStarConfig,
    ) -> Option<Self> {
        let goal_layers = layer_indices(&config.layers, target);
        let (gx, gy) = snap(target.point, config.grid_step);
        let goal = GridNode {
            gx,
            gy,
            layer_idx: goal_layers.first().copied()?,
        };
        let wire = Probe {
            width: config.trace_width,
            clearance: config.clearance,
            net_id,
        };
        Some(Self {
            spatial,
            config,
            start,
            target,
            wire,
            via: Probe {
                width: config.via_diameter,
                ..wire
            },
            goal,
            goal_layers,
            open_set: BinaryHeap::new(),
            g_score: HashMap::new(),
            came_from: HashMap::new(),
            closed: HashSet::new(),
        })
    }

    fn at(&self, n: GridNode) -> [f64; 2] {
        let step = self.config.grid_step;
        [n.gx as f64 * step, n.gy as f64 * step]
    }

    fn wire_clear(&self, layer: usize, a: [f64; 2], b: [f64; 2]) -> bool {
        self.spatial
            .is_segment_clear(&self.config.layers[layer], a, b, self.wire)
    }

    /// Opens the start node on every start layer it can leave the pad from.
    pub(super) fn seed(&mut self) {
        let (sgx, sgy) = snap(self.start.point, self.config.grid_step);
        for layer_idx in layer_indices(&self.config.layers, self.start) {
            let node = GridNode {
                gx: sgx,
                gy: sgy,
                layer_idx,
            };
            if self.wire_clear(layer_idx, self.start.point, self.at(node)) {
                self.g_score.insert(node, 0.0);
                self.open_set.push(SearchState {
                    cost: heuristic(node, self.goal, self.config.grid_step),
                    node,
                });
            }
        }
    }

    /// Expands nodes until the goal connects to the target pad or the budget runs out.
    pub(super) fn run(mut self) -> Option<RoutePath> {
        while let Some(SearchState { node, .. }) = self.open_set.pop() {
            if !self.closed.insert(node) {
                continue;
            }
            if self.closed.len() > MAX_EXPANSIONS {
                break;
            }
            let current_p = self.at(node);
            if node.gx == self.goal.gx
                && node.gy == self.goal.gy
                && self.goal_layers.contains(&node.layer_idx)
                && self.wire_clear(node.layer_idx, current_p, self.target.point)
            {
                let frame = PathFrame {
                    step: self.config.grid_step,
                    layers: &self.config.layers,
                    start: self.start,
                    target: self.target,
                };
                return Some(reconstruct_path(node, &self.came_from, &frame));
            }
            let current_g = self.g_score[&node];
            self.expand_wires(node, current_p, current_g);
            self.expand_vias(node, current_p, current_g);
        }
        None
    }

    fn expand_wires(&mut self, node: GridNode, current_p: [f64; 2], current_g: f64) {
        let step = self.config.grid_step;
        for &(dx, dy) in &DIRS {
            let next = GridNode {
                gx: node.gx + dx,
                gy: node.gy + dy,
                layer_idx: node.layer_idx,
            };
            if self.closed.contains(&next)
                || !self.wire_clear(node.layer_idx, current_p, self.at(next))
            {
                continue;
            }
            let dist = if dx != 0 && dy != 0 {
                step * std::f64::consts::SQRT_2
            } else {
                step
            };
            self.relax(node, next, current_g + dist);
        }
    }

    fn expand_vias(&mut self, node: GridNode, current_p: [f64; 2], current_g: f64) {
        let layer_count = self.config.layers.len();
        if layer_count < 2 {
            return;
        }
        let via_clear = self.config.layers.iter().all(|layer| {
            self.spatial
                .is_segment_clear(layer, current_p, current_p, self.via)
        });
        if !via_clear {
            return;
        }
        for layer_idx in (0..layer_count).filter(|&i| i != node.layer_idx) {
            self.relax(
                node,
                GridNode { layer_idx, ..node },
                current_g + self.config.via_cost,
            );
        }
    }

    fn relax(&mut self, from: GridNode, next: GridNode, tentative_g: f64) {
        if tentative_g < *self.g_score.get(&next).unwrap_or(&f64::INFINITY) {
            self.g_score.insert(next, tentative_g);
            self.came_from.insert(next, from);
            self.open_set.push(SearchState {
                cost: tentative_g + heuristic(next, self.goal, self.config.grid_step),
                node: next,
            });
        }
    }
}
