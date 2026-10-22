//! Autorouting entry point: Circuit JSON in, routed Circuit JSON out.

mod connectivity;
mod index;
mod json;
mod output;
mod passes;
mod trace;

use serde_json::Value;

use super::astar::{AStarConfig, Terminal};
use index::{build_index, NetTerminals};
use output::emit_traces;
use passes::route_jobs;

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
    pub start: Terminal,
    pub target: Terminal,
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
    let config = astar_config(options);
    let (base, net_terminals) = build_index(&elements, &config)?;
    let jobs = connection_jobs(&net_terminals);
    let solved = route_jobs(&jobs, &base, &config, options.max_iterations);
    let emitted = emit_traces(&jobs, solved, options, &config);

    if !options.dry_run {
        elements.extend(emitted.elements);
    }

    let output_json = serde_json::to_string_pretty(&elements)
        .map_err(|e| format!("failed to serialize routed JSON: {e}"))?;

    Ok(RouteResult {
        total_nets: jobs.len(),
        routed_nets: emitted.routed_nets,
        unrouted_nets: emitted.unrouted_nets,
        generated_traces: emitted.routed_nets,
        generated_vias: emitted.generated_vias,
        output_json,
    })
}

fn astar_config(options: &RouteOptions) -> AStarConfig {
    AStarConfig {
        trace_width: options.trace_width,
        clearance: options.clearance,
        layers: if options.layers > 1 {
            vec!["top".to_string(), "bottom".to_string()]
        } else {
            vec!["top".to_string()]
        },
        ..Default::default()
    }
}

/// One job per consecutive terminal pair of every net.
fn connection_jobs(net_terminals: &NetTerminals) -> Vec<ConnectionJob> {
    let mut jobs = Vec::new();
    for (net_id, terminals) in net_terminals {
        for pair in terminals.windows(2) {
            jobs.push(ConnectionJob {
                net_id: net_id.clone(),
                start: pair[0].clone(),
                target: pair[1].clone(),
            });
        }
    }
    jobs
}
