//! Rip-up and reroute: frees a blocked connection by moving a neighbour.

use super::super::astar::{find_path, AStarConfig, RoutePath, RouteStepKind};
use super::super::spatial::{segment_distance, SpatialRoutingIndex};
use super::trace::insert_path;
use super::ConnectionJob;

const NEIGHBOUR_RADIUS: f64 = 6.0;
const MAX_ROUNDS: usize = 4;

fn step_point(step: &RouteStepKind) -> [f64; 2] {
    match step {
        RouteStepKind::Wire { x, y, .. } | RouteStepKind::Via { x, y, .. } => [*x, *y],
    }
}

fn distance_to_path(point: [f64; 2], path: &RoutePath) -> f64 {
    let points: Vec<[f64; 2]> = path.steps.iter().map(step_point).collect();
    let nearest_vertex = points
        .iter()
        .map(|p| segment_distance(point, point, *p, *p))
        .fold(f64::INFINITY, f64::min);
    points
        .windows(2)
        .map(|pair| segment_distance(point, point, pair[0], pair[1]))
        .fold(nearest_vertex, f64::min)
}

fn neighbours(
    job: &ConnectionJob,
    jobs: &[ConnectionJob],
    paths: &[Option<RoutePath>],
) -> Vec<usize> {
    let mut near: Vec<(f64, usize)> = paths
        .iter()
        .enumerate()
        .filter_map(|(i, path)| {
            let path = path.as_ref()?;
            let d = distance_to_path(job.start.point, path)
                .min(distance_to_path(job.target.point, path));
            (d <= NEIGHBOUR_RADIUS && jobs[i].net_id != job.net_id).then_some((d, i))
        })
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    near.into_iter().map(|(_, i)| i).collect()
}

struct Problem<'a> {
    jobs: &'a [ConnectionJob],
    base: &'a SpatialRoutingIndex,
    config: &'a AStarConfig,
}

impl Problem<'_> {
    fn index_without(&self, paths: &[Option<RoutePath>], skip: usize) -> SpatialRoutingIndex {
        let mut index = self.base.clone();
        for (i, path) in paths.iter().enumerate().filter(|(i, _)| *i != skip) {
            if let Some(path) = path {
                insert_path(&mut index, &self.jobs[i].net_id, path, self.config);
            }
        }
        index
    }

    fn swap_in(
        &self,
        paths: &[Option<RoutePath>],
        failed: usize,
        victim: usize,
    ) -> Option<(RoutePath, RoutePath)> {
        let mut index = self.index_without(paths, victim);
        let unblocked = route(&self.jobs[failed], &mut index, self.config)?;
        let moved = route(&self.jobs[victim], &mut index, self.config)?;
        Some((unblocked, moved))
    }
}

fn route(
    job: &ConnectionJob,
    index: &mut SpatialRoutingIndex,
    config: &AStarConfig,
) -> Option<RoutePath> {
    let path = find_path(&job.start, &job.target, &job.net_id, index, config)?;
    insert_path(index, &job.net_id, &path, config);
    Some(path)
}

fn rip_up_round(
    jobs: &[ConnectionJob],
    base: &SpatialRoutingIndex,
    config: &AStarConfig,
    paths: &mut [Option<RoutePath>],
) -> usize {
    let problem = Problem { jobs, base, config };
    let failed_jobs: Vec<usize> = (0..jobs.len()).filter(|&j| paths[j].is_none()).collect();
    let mut fixed = 0;
    for failed in failed_jobs {
        for victim in neighbours(&jobs[failed], jobs, paths) {
            if let Some((unblocked, moved)) = problem.swap_in(paths, failed, victim) {
                paths[failed] = Some(unblocked);
                paths[victim] = Some(moved);
                fixed += 1;
                break;
            }
        }
    }
    fixed
}

/// Routes failed connections by first lifting a nearby routed one and
/// rerouting it afterwards; a swap is kept only when both connections route.
/// Repeats while a round still fixes something.
pub(crate) fn rip_up_and_reroute(
    jobs: &[ConnectionJob],
    base: &SpatialRoutingIndex,
    config: &AStarConfig,
    paths: &mut [Option<RoutePath>],
) {
    for _ in 0..MAX_ROUNDS {
        if rip_up_round(jobs, base, config, paths) == 0 {
            break;
        }
    }
}
