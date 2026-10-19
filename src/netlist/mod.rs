//! Step 2.5 — Netlist connectivity graph.
//!
//! Lowers the raw [`Circuit`] AST into an indexed electrical graph.
//!
//! # Architecture: Deep Module with Trivial Defaults
//!
//! The raw Circuit AST is a flat, heterogeneous list of elements where
//! electrical connectivity is scattered across `source_port.net_id`,
//! `source_pad.net_id`, and `source_trace.connected_source_port_ids`.
//!
//! Querying connectivity directly on [`Circuit`] requires repeated O(N)
//! traversals and ad-hoc string deduplication. `Netlist` lowers the AST
//! once into an indexed graph:
//!
//! - **Equipotential nodes**: nets resolved by ID and human-readable name;
//! - **Unified endpoints**: ports and pads unified under [`Endpoint`];
//! - **Trace routing**: traces propagate endpoints across connected nets;
//! - **Fail-closed tracking**: undefined ("ghost") nets and anonymous
//!   endpoints are preserved, upholding the linter's fail-closed contract.
//!
//! # Ergonomics: Zero-Boilerplate Queries
//!
//! Designed around: *"Optimize for the most common caller — make the default case trivial."*
//!
//! Rules and linter passes can query connectivity using string literals,
//! AST elements, or graph nodes interchangeably via [`NetQuery`]:
//!
//! ```ignore
//! let netlist = Netlist::from_circuit(&circuit);
//!
//! // Query endpoints directly by name or net_id: returns &[Endpoint]
//! for ep in netlist.endpoints("GND") {
//!     println!("pin: {:?}", ep.id());
//! }
//!
//! // Check floating status with zero unwrap/boilerplate:
//! if netlist.is_floating("GND") {
//!     // ...
//! }
//! ```

use std::collections::{HashMap, HashSet};

mod builder;
mod endpoint;
mod node;
mod query;
#[cfg(test)]
mod tests;

pub use endpoint::{Endpoint, EndpointKind};
pub use node::NetNode;
pub use query::NetQuery;

use crate::parser::Circuit;

/// The lowered, indexed electrical connectivity graph of a [`Circuit`].
#[derive(Debug, Clone, Default)]
pub struct Netlist {
    pub(crate) nets: Vec<NetNode>,
    pub(crate) by_id: HashMap<String, usize>,
    pub(crate) by_name: HashMap<String, usize>,
    pub(crate) port_to_net: HashMap<String, usize>,
    pub(crate) pad_to_net: HashMap<String, usize>,
    pub(crate) endpoint_ids_per_net: Vec<HashSet<String>>,
}

impl Netlist {
    /// Lowers a raw [`Circuit`] AST into an indexed electrical graph.
    pub fn from_circuit(circuit: &Circuit) -> Self {
        builder::build_netlist(circuit)
    }

    /// Looks up a net node by query (`"GND"`, `"n1"`, `&Net`, etc.).
    pub fn net(&self, query: impl NetQuery) -> Option<&NetNode> {
        query.resolve_index(self).map(|idx| &self.nets[idx])
    }

    /// Queries endpoints for a net. Returns an empty slice `&[]` if not found.
    pub fn endpoints(&self, query: impl NetQuery) -> &[Endpoint] {
        match query.resolve_index(self) {
            Some(idx) => &self.nets[idx].endpoints,
            None => &[],
        }
    }

    /// Returns true if the net has fewer than 2 endpoints (or does not exist).
    pub fn is_floating(&self, query: impl NetQuery) -> bool {
        match query.resolve_index(self) {
            Some(idx) => self.nets[idx].is_floating(),
            None => true,
        }
    }

    /// Iterator over all nets in deterministic order.
    pub fn nets(&self) -> impl Iterator<Item = &NetNode> {
        self.nets.iter()
    }

    /// Iterator over floating nets (fewer than 2 endpoints).
    pub fn floating_nets(&self) -> impl Iterator<Item = &NetNode> {
        self.nets.iter().filter(|n| n.is_floating())
    }

    /// Iterator over referenced but undefined nets (ghost nets).
    pub fn ghost_nets(&self) -> impl Iterator<Item = &NetNode> {
        self.nets.iter().filter(|n| n.is_ghost())
    }

    /// Returns true if two endpoints share the same electrical net.
    pub fn are_connected(&self, endpoint_a: &str, endpoint_b: &str) -> bool {
        let net_a = self
            .port_to_net
            .get(endpoint_a)
            .or_else(|| self.pad_to_net.get(endpoint_a));
        let net_b = self
            .port_to_net
            .get(endpoint_b)
            .or_else(|| self.pad_to_net.get(endpoint_b));
        match (net_a, net_b) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }
}
