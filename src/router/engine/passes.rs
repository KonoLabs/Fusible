//! Multi-pass routing of connection jobs against a shared obstacle index.

use super::super::astar::{find_path, AStarConfig, RoutePath};
use super::super::spatial::SpatialRoutingIndex;
use super::ripup::rip_up_and_reroute;
use super::trace::insert_path;
use super::ConnectionJob;

/// Routes jobs one at a time so every finished path becomes an obstacle;
/// failed jobs move to the front for the next of up to `max_iterations`
/// passes, and the pass routing the most jobs wins.
pub(super) fn route_jobs(
    jobs: &[ConnectionJob],
    base: &SpatialRoutingIndex,
    config: &AStarConfig,
    max_iterations: usize,
) -> Vec<Option<RoutePath>> {
    let mut order: Vec<usize> = (0..jobs.len()).collect();
    let mut best: Vec<Option<RoutePath>> = vec![None; jobs.len()];
    let mut best_count = None;

    for _ in 0..max_iterations.max(1) {
        let mut index = base.clone();
        let mut paths: Vec<Option<RoutePath>> = vec![None; jobs.len()];
        for &j in &order {
            let job = &jobs[j];
            if let Some(path) = find_path(&job.start, &job.target, &job.net_id, &index, config) {
                insert_path(&mut index, &job.net_id, &path, config);
                paths[j] = Some(path);
            }
        }
        let routed = paths.iter().filter(|p| p.is_some()).count();
        if best_count.is_none_or(|b| routed > b) {
            best_count = Some(routed);
            best = paths.clone();
        }
        if routed == jobs.len() {
            break;
        }
        let (failed, done): (Vec<usize>, Vec<usize>) =
            order.iter().partition(|&&j| paths[j].is_none());
        order = failed.into_iter().chain(done).collect();
    }
    rip_up_and_reroute(jobs, base, config, &mut best);
    best
}
