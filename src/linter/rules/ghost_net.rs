//! `ghost_net`: every referenced net must be defined.

use std::collections::{HashMap, HashSet};

use crate::linter::{Diagnostic, Rule};
use crate::parser::Circuit;

/// A `net_id` referenced by ports or pads but defined by no `source_net`.
///
/// The rule loops over ports and pads, not over `nets()`: a reference to
/// copper that does not exist would be invisible to a check built only on
/// defined nets.
pub struct GhostNetRule;

/// Every net id referenced by a port, pad, or trace, ports first.
fn net_references(circuit: &Circuit) -> impl Iterator<Item = &str> {
    circuit
        .ports()
        .filter_map(|p| p.net_id.as_deref())
        .chain(circuit.pads().filter_map(|p| p.net_id.as_deref()))
        .chain(
            circuit
                .traces()
                .flat_map(|t| t.connected_source_net_ids.iter().map(String::as_str)),
        )
}

/// Reference count per net id. Built in a first pass because HashMap
/// iteration is randomized and `run()` promises a deterministic diagnostic
/// order — the emission order comes from `net_references`.
fn reference_counts(circuit: &Circuit) -> HashMap<&str, usize> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for id in net_references(circuit) {
        *counts.entry(id).or_insert(0) += 1;
    }
    counts
}

fn ghost_diagnostic(id: &str, count: usize) -> Diagnostic {
    Diagnostic {
        rule: "ghost_net".to_string(),
        message: format!(
            "net \"{id}\" is referenced by {count} connection(s) \
             but no source_net defines it"
        ),
        help: Some(
            "add the missing source_net element, or fix the net_id \
             reference on the port/pad"
                .to_string(),
        ),
    }
}

impl Rule for GhostNetRule {
    fn name(&self) -> &'static str {
        "ghost_net"
    }

    fn check(&self, circuit: &Circuit) -> Vec<Diagnostic> {
        let defined: HashSet<&str> = circuit.nets().filter_map(|n| n.net_id.as_deref()).collect();
        let counts = reference_counts(circuit);

        let mut seen: HashSet<&str> = HashSet::new();
        net_references(circuit)
            .filter(|id| !defined.contains(id) && seen.insert(id))
            .map(|id| ghost_diagnostic(id, counts[id]))
            .collect()
    }
}
