//! Net registration and endpoint attachment shared by every lowering step.

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

pub(super) fn register_net(
    netlist: &mut Netlist,
    net_id: &str,
    name: Option<&str>,
    is_defined: bool,
) -> usize {
    match netlist.by_id.get(net_id).copied() {
        Some(idx) => {
            promote_defined_net(netlist, idx, name, is_defined);
            idx
        }
        None => insert_new_net(netlist, net_id, name, is_defined),
    }
}

fn promote_defined_net(netlist: &mut Netlist, idx: usize, name: Option<&str>, is_defined: bool) {
    if !is_defined {
        return;
    }
    let node = &mut netlist.nets[idx];
    node.is_defined = true;
    if node.name.is_none() {
        node.name = name.map(str::to_string);
    }
}

fn insert_new_net(
    netlist: &mut Netlist,
    net_id: &str,
    name: Option<&str>,
    is_defined: bool,
) -> usize {
    let idx = netlist.nets.len();
    netlist.nets.push(NetNode {
        id: net_id.to_string(),
        name: name.map(str::to_string),
        endpoints: Vec::new(),
        is_defined,
        reference_count: 0,
    });
    netlist.endpoint_ids_per_net.push(HashSet::new());
    netlist.by_id.insert(net_id.to_string(), idx);
    idx
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
