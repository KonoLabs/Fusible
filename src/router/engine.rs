use rayon::prelude::*;
use rstar::AABB;
use serde_json::{json, Value};

use super::astar::{find_path, AStarConfig, RoutePath, RouteStepKind};
use super::spatial::{ObstacleKind, SpatialObstacle, SpatialRoutingIndex};

#[derive(Clone, Debug)]
pub struct RouteOptions {
    pub clearance: f64,
    pub trace_width: f64,
    pub layers: u8,
    pub max_iterations: usize,
    pub dry_run: bool,
}

impl Default for RouteOptions {
    fn default() -> Self {
        Self {
            clearance: 0.15,
            trace_width: 0.2,
            layers: 2,
            max_iterations: 10,
            dry_run: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionJob {
    pub net_id: String,
    pub start: [f64; 2],
    pub target: [f64; 2],
}

#[derive(Debug)]
pub struct RouteResult {
    pub total_nets: usize,
    pub routed_nets: usize,
    pub unrouted_nets: Vec<String>,
    pub generated_traces: usize,
    pub generated_vias: usize,
    pub output_json: String,
}

pub fn route_circuit(circuit_json: &str, options: &RouteOptions) -> Result<RouteResult, String> {
    let mut elements: Vec<Value> =
        serde_json::from_str(circuit_json).map_err(|e| format!("invalid Circuit JSON: {e}"))?;

    let mut spatial = SpatialRoutingIndex::new();
    let mut net_pads: std::collections::HashMap<String, Vec<[f64; 2]>> =
        std::collections::HashMap::new();

    let mut obstacle_id = 0;
    for el in &elements {
        let typ = el.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if typ == "pcb_smtpad" || typ == "source_pad" {
            let x = el.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let y = el.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let w = el.get("width").and_then(|v| v.as_f64()).unwrap_or(1.0);
            let h = el.get("height").and_then(|v| v.as_f64()).unwrap_or(1.0);
            let layer = el
                .get("layer")
                .and_then(|v| v.as_str())
                .unwrap_or("top")
                .to_string();
            let net_id = el
                .get("net_id")
                .or_else(|| el.get("source_net_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if let Some(ref nid) = net_id {
                net_pads.entry(nid.clone()).or_default().push([x, y]);
            }

            let half_w = w / 2.0;
            let half_h = h / 2.0;
            spatial.insert_obstacle(
                &layer,
                SpatialObstacle {
                    id: obstacle_id,
                    net_id: net_id.clone(),
                    layer: layer.clone(),
                    kind: ObstacleKind::Pad {
                        net_id: net_id.unwrap_or_default(),
                    },
                    aabb: AABB::from_corners([x - half_w, y - half_h], [x + half_w, y + half_h]),
                },
            );
            obstacle_id += 1;
        }
    }

    let mut jobs = Vec::new();
    for (net_id, pads) in &net_pads {
        if pads.len() < 2 {
            continue;
        }
        for i in 0..pads.len() - 1 {
            jobs.push(ConnectionJob {
                net_id: net_id.clone(),
                start: pads[i],
                target: pads[i + 1],
            });
        }
    }

    let config = AStarConfig {
        trace_width: options.trace_width,
        clearance: options.clearance,
        layers: if options.layers > 1 {
            vec!["top".to_string(), "bottom".to_string()]
        } else {
            vec!["top".to_string()]
        },
        ..Default::default()
    };

    let total_nets = jobs.len();
    let spatial_ref = &spatial;
    let config_ref = &config;

    let solved_paths: Vec<(ConnectionJob, Option<RoutePath>)> = jobs
        .into_par_iter()
        .map(|job| {
            let path = find_path(job.start, job.target, &job.net_id, spatial_ref, config_ref);
            (job, path)
        })
        .collect();

    let mut routed_nets = 0;
    let mut unrouted_nets = Vec::new();
    let mut generated_traces = 0;
    let mut generated_vias = 0;

    let mut new_elements = Vec::new();

    for (job, maybe_path) in solved_paths {
        if let Some(path) = maybe_path {
            routed_nets += 1;
            generated_traces += 1;

            let mut route_array = Vec::new();
            for step in &path.steps {
                match step {
                    RouteStepKind::Wire { layer, x, y } => {
                        route_array.push(json!({
                            "route_type": "wire",
                            "x": x,
                            "y": y,
                            "width": options.trace_width,
                            "layer": layer
                        }));
                    }
                    RouteStepKind::Via { x, y } => {
                        generated_vias += 1;
                        route_array.push(json!({
                            "route_type": "via",
                            "x": x,
                            "y": y,
                            "from_layer": "top",
                            "to_layer": "bottom"
                        }));
                    }
                }
            }

            new_elements.push(json!({
                "type": "pcb_trace",
                "pcb_trace_id": format!("pcb_trace_auto_{routed_nets}"),
                "net_id": job.net_id,
                "route": route_array
            }));
        } else {
            unrouted_nets.push(job.net_id);
        }
    }

    if !options.dry_run {
        elements.extend(new_elements);
    }

    let output_json = serde_json::to_string_pretty(&elements)
        .map_err(|e| format!("failed to serialize routed JSON: {e}"))?;

    Ok(RouteResult {
        total_nets,
        routed_nets,
        unrouted_nets,
        generated_traces,
        generated_vias,
        output_json,
    })
}
