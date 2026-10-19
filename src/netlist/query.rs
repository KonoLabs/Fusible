//! Ergonomic net querying trait.

use super::node::NetNode;
use super::Netlist;
use crate::parser::Net;

/// Trait abstracting net lookups so callers can query by name, ID, or AST reference.
pub trait NetQuery {
    /// Resolves the index into `Netlist::nets`.
    fn resolve_index(&self, netlist: &Netlist) -> Option<usize>;
}

impl<T: AsRef<str> + ?Sized> NetQuery for T {
    fn resolve_index(&self, netlist: &Netlist) -> Option<usize> {
        let s = self.as_ref();
        netlist
            .by_name
            .get(s)
            .or_else(|| netlist.by_id.get(s))
            .copied()
    }
}

impl NetQuery for NetNode {
    fn resolve_index(&self, netlist: &Netlist) -> Option<usize> {
        netlist.by_id.get(&self.id).copied()
    }
}

impl NetQuery for &NetNode {
    fn resolve_index(&self, netlist: &Netlist) -> Option<usize> {
        netlist.by_id.get(&self.id).copied()
    }
}

impl NetQuery for Net {
    fn resolve_index(&self, netlist: &Netlist) -> Option<usize> {
        if let Some(id) = self.net_id.as_deref() {
            if let Some(idx) = netlist.by_id.get(id) {
                return Some(*idx);
            }
        }
        self.name
            .as_deref()
            .and_then(|name| netlist.by_name.get(name).copied())
    }
}

impl NetQuery for &Net {
    fn resolve_index(&self, netlist: &Netlist) -> Option<usize> {
        (*self).resolve_index(netlist)
    }
}
