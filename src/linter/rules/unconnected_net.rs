//! `unconnected_net`: a net must not be left floating.

use crate::linter::{Diagnostic, Rule};
use crate::parser::Circuit;

/// Minimum number of endpoints for a net to count as connected.
const MIN_CONNECTIONS: usize = 2;

/// A net tied to fewer than two pads/ports is a routing omission: on the
/// real board that signal goes nowhere. The fuse blows before the bug can
/// reach the firmware.
///
/// Note this checks *endpoints*, not copper: two ports on the same net with
/// no `source_trace` between them still pass (a `unrouted_net` rule is
/// phase-2 material).
///
/// Nets without an identifier are not silently ignored here: they are
/// reported by `MalformedElementRule`, which `Linter::default()` registers
/// alongside this rule. Skipping is safe only because the registry speaks
/// for the skipped element — that pairing is the fail-closed contract.
pub struct UnconnectedNetRule;

/// Port IDs attached to `net_id` through traces.
fn trace_endpoints<'a>(circuit: &'a Circuit, net_id: &'a str) -> impl Iterator<Item = &'a str> {
    circuit
        .traces()
        .filter(move |trace| trace.connected_source_net_ids.iter().any(|id| id == net_id))
        .flat_map(|trace| trace.connected_source_port_ids.iter().map(String::as_str))
}

/// Number of ports + pads attached to `net_id`.
fn endpoint_count(circuit: &Circuit, net_id: &str) -> usize {
    let mut endpoints = std::collections::HashSet::new();
    let mut anonymous = 0usize;

    let port_pads =
        circuit
            .ports()
            .filter_map(|p| (p.net_id.as_deref() == Some(net_id)).then_some(p.port_id.as_deref()))
            .chain(circuit.pads().filter_map(|p| {
                (p.net_id.as_deref() == Some(net_id)).then_some(p.pad_id.as_deref())
            }));

    for opt_id in port_pads {
        match opt_id {
            Some(id) => {
                endpoints.insert(id);
            }
            None => anonymous += 1,
        }
    }

    endpoints.extend(trace_endpoints(circuit, net_id));
    endpoints.len() + anonymous
}

impl Rule for UnconnectedNetRule {
    fn name(&self) -> &'static str {
        "unconnected_net"
    }

    fn check(&self, circuit: &Circuit) -> Vec<Diagnostic> {
        let mut violations = Vec::new();

        for net in circuit.nets() {
            let Some(net_id) = net.net_id.as_deref() else {
                continue;
            };

            let endpoints = endpoint_count(circuit, net_id);
            if endpoints >= MIN_CONNECTIONS {
                continue;
            }

            let label = net.name.as_deref().unwrap_or(net_id);
            violations.push(Diagnostic {
                rule: self.name().to_string(),
                message: format!(
                    "net \"{label}\" has only {endpoints} connection(s) \
                     (minimum {MIN_CONNECTIONS}): it is floating"
                ),
                help: Some(
                    "every net must connect at least two pads/ports — \
                     complete the routing in tscircuit"
                        .to_string(),
                ),
            });
        }

        violations
    }
}
