//! Step 2.5 — Netlist connectivity graph.
//!
//! Lowers the raw [`Circuit`] AST into an indexed electrical graph.
//!
//! The raw Circuit AST is a flat, heterogeneous list of elements where
//! electrical connectivity is scattered across `source_port.net_id`,
//! `source_trace.connected_source_port_ids`, and the
//! `pcb_smtpad.pcb_port_id` -> `pcb_port.source_port_id` chain.
//! `Netlist` lowers the AST once into an indexed graph:
//!
//! - **Equipotential nodes**: nets resolved by ID;
//! - **Unified endpoints**: ports and pads unified under `Endpoint`, a pad
//!   linked to a port counted once, as that port;
//! - **Trace routing**: nets, ports and traces join a union-find, and one
//!   net is cut per connected component, so a trace tying two nets merges
//!   them and a short between declared nets is recorded;
//! - **Fail-closed tracking**: undefined ("ghost") nets, anonymous
//!   endpoints, and references to elements that do not exist are preserved
//!   for the linter to report.

use std::collections::{HashMap, HashSet};

mod builder;
mod connectivity;
mod endpoint;
mod groups;
mod labels;
#[cfg(test)]
mod merge_tests;
mod node;
mod pads;
mod registry;
#[cfg(test)]
mod tests;
mod union_find;

pub use node::NetNode;

use crate::parser::Circuit;

/// A reference from one element to another element that does not exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanglingRef {
    /// The referencing element, e.g. `source_trace "t1"`.
    pub owner: String,
    /// The missing element, e.g. `source_port "U9.1"`.
    pub target: String,
}

/// Two or more differently named declared nets that one trace ties together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Short {
    /// Names of the merged declared nets, sorted.
    pub nets: Vec<String>,
    /// The first trace in the merged group, e.g. `source_trace "t1"`.
    pub trace: String,
}

/// The lowered, indexed electrical connectivity graph of a [`Circuit`].
#[derive(Debug, Clone, Default)]
pub struct Netlist {
    pub(crate) nets: Vec<NetNode>,
    pub(crate) by_id: HashMap<String, usize>,
    pub(crate) port_to_net: HashMap<String, usize>,
    pub(crate) pad_to_net: HashMap<String, usize>,
    pub(crate) endpoint_ids_per_net: Vec<HashSet<String>>,
    pub(crate) dangling_refs: Vec<DanglingRef>,
    pub(crate) ghosts: Vec<NetNode>,
    pub(crate) shorts: Vec<Short>,
}

impl Netlist {
    /// Lowers a raw [`Circuit`] AST into an indexed electrical graph.
    pub fn from_circuit(circuit: &Circuit) -> Self {
        builder::build_netlist(circuit)
    }

    /// Iterator over floating nets (fewer than 2 endpoints).
    pub fn floating_nets(&self) -> impl Iterator<Item = &NetNode> {
        self.nets.iter().filter(|n| n.is_floating())
    }

    /// Iterator over referenced but undefined nets (ghost nets).
    pub fn ghost_nets(&self) -> impl Iterator<Item = &NetNode> {
        self.ghosts.iter()
    }

    /// Declared nets that traces merged into one.
    pub fn shorts(&self) -> &[Short] {
        &self.shorts
    }

    /// References to ports, PCB ports, or pads that no element defines.
    pub fn dangling_refs(&self) -> &[DanglingRef] {
        &self.dangling_refs
    }
}
