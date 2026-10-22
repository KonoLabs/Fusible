//! Grid nodes, open-set ordering, and the octile distance heuristic.

use std::cmp::Ordering;

use super::Terminal;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(super) struct GridNode {
    pub gx: i32,
    pub gy: i32,
    pub layer_idx: usize,
}

#[derive(Copy, Clone)]
pub(super) struct SearchState {
    pub cost: f64,
    pub node: GridNode,
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

/// Octile distance between two grid nodes, in millimeters.
pub(super) fn heuristic(a: GridNode, b: GridNode, step: f64) -> f64 {
    let dx = (a.gx - b.gx).abs() as f64 * step;
    let dy = (a.gy - b.gy).abs() as f64 * step;
    dx.min(dy) * std::f64::consts::SQRT_2 + (dx - dy).abs()
}

/// Indices into `layers` of the layers the terminal exists on.
pub(super) fn layer_indices(layers: &[String], terminal: &Terminal) -> Vec<usize> {
    (0..layers.len())
        .filter(|&i| terminal.layers.contains(&layers[i]))
        .collect()
}

/// The grid cell nearest to `p`.
pub(super) fn snap(p: [f64; 2], step: f64) -> (i32, i32) {
    ((p[0] / step).round() as i32, (p[1] / step).round() as i32)
}
