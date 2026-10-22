//! Rendering routed paths as `pcb_trace` elements.

use serde_json::{json, Value};

use super::super::astar::{AStarConfig, RoutePath, RouteStepKind};
use super::{ConnectionJob, RouteOptions};

/// New `pcb_trace` elements and the counts they contribute to the result.
#[derive(Default)]
pub(super) struct Emitted {
    pub elements: Vec<Value>,
    pub routed_nets: usize,
    pub unrouted_nets: Vec<String>,
    pub generated_vias: usize,
}

/// Renders every solved job as a `pcb_trace`, numbering traces in job order.
pub(super) fn emit_traces(
    jobs: &[ConnectionJob],
    solved: Vec<Option<RoutePath>>,
    options: &RouteOptions,
    config: &AStarConfig,
) -> Emitted {
    let mut out = Emitted::default();
    for (job, maybe_path) in jobs.iter().zip(solved) {
        let Some(path) = maybe_path else {
            out.unrouted_nets.push(job.net_id.clone());
            continue;
        };
        out.routed_nets += 1;
        let route_array: Vec<Value> = path
            .steps
            .iter()
            .map(|step| step_json(step, options, config, &mut out.generated_vias))
            .collect();
        out.elements.push(json!({
            "type": "pcb_trace",
            "pcb_trace_id": format!("pcb_trace_auto_{}", out.routed_nets),
            "net_id": job.net_id,
            "route": route_array
        }));
    }
    out
}

fn step_json(
    step: &RouteStepKind,
    options: &RouteOptions,
    config: &AStarConfig,
    generated_vias: &mut usize,
) -> Value {
    match step {
        RouteStepKind::Wire { layer, x, y } => json!({
            "route_type": "wire",
            "x": x,
            "y": y,
            "width": options.trace_width,
            "layer": layer
        }),
        RouteStepKind::Via {
            x,
            y,
            from_layer,
            to_layer,
        } => {
            *generated_vias += 1;
            json!({
                "route_type": "via",
                "x": x,
                "y": y,
                "from_layer": from_layer,
                "to_layer": to_layer,
                "outer_diameter": config.via_diameter,
                "hole_diameter": config.via_hole_diameter
            })
        }
    }
}
