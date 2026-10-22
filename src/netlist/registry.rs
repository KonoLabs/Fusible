//! Net insertion and endpoint attachment shared by every lowering step.

use std::collections::HashSet;

use super::endpoint::Endpoint;
use super::node::NetNode;
use super::Netlist;
use crate::parser::Port;

pub(super) fn attach_port(netlist: &mut Netlist, idx: usize, port: &Port) {
    attach_endpoint(netlist, idx, Endpoint::port(port));
    if let Some(id) = port.port_id.as_deref() {
        netlist.port_to_net.insert(id.to_string(), idx);
    }
}

pub(super) fn insert_net(netlist: &mut Netlist, node: NetNode) -> usize {
    netlist.nets.push(node);
    netlist.endpoint_ids_per_net.push(HashSet::new());
    netlist.nets.len() - 1
}

pub(super) fn attach_endpoint(netlist: &mut Netlist, net_idx: usize, endpoint: Endpoint) {
    let is_new = match endpoint.id.as_deref() {
        Some(id) => netlist.endpoint_ids_per_net[net_idx].insert(id.to_string()),
        None => true,
    };
    if is_new {
        netlist.nets[net_idx].endpoints.push(endpoint);
    }
}
