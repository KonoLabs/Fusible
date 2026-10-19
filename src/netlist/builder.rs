//! AST lowering into the Netlist connectivity graph.

use std::collections::HashSet;

use super::endpoint::Endpoint;
use super::node::NetNode;
use super::Netlist;
use crate::parser::Circuit;

pub fn build_netlist(circuit: &Circuit) -> Netlist {
    let mut netlist = Netlist::default();
    index_defined_nets(&mut netlist, circuit);
    index_ports_and_pads(&mut netlist, circuit);
    index_traces(&mut netlist, circuit);
    netlist
}

fn index_defined_nets(netlist: &mut Netlist, circuit: &Circuit) {
    for net in circuit.nets() {
        if let Some(net_id) = net.net_id.as_deref() {
            register_net(netlist, net_id, net.name.as_deref(), true);
        }
    }
}

fn index_ports_and_pads(netlist: &mut Netlist, circuit: &Circuit) {
    for port in circuit.ports() {
        let Some(net_id) = port.net_id.as_deref() else {
            continue;
        };
        let idx = register_net(netlist, net_id, None, false);
        netlist.nets[idx].reference_count += 1;
        attach_endpoint(
            netlist,
            idx,
            Endpoint::port(port.port_id.clone(), port.name.clone()),
        );
        if let Some(id) = port.port_id.as_deref() {
            netlist.port_to_net.insert(id.to_string(), idx);
        }
    }
    for pad in circuit.pads() {
        let Some(net_id) = pad.net_id.as_deref() else {
            continue;
        };
        let idx = register_net(netlist, net_id, None, false);
        netlist.nets[idx].reference_count += 1;
        attach_endpoint(netlist, idx, Endpoint::pad(pad.pad_id.clone()));
        if let Some(id) = pad.pad_id.as_deref() {
            netlist.pad_to_net.insert(id.to_string(), idx);
        }
    }
}

fn index_traces(netlist: &mut Netlist, circuit: &Circuit) {
    for trace in circuit.traces() {
        for net_id in &trace.connected_source_net_ids {
            let idx = register_net(netlist, net_id, None, false);
            netlist.nets[idx].reference_count += 1;

            if let Some(trace_id) = trace.trace_id.as_deref() {
                netlist.nets[idx].trace_ids.push(trace_id.to_string());
            }

            for port_id in &trace.connected_source_port_ids {
                let endpoint = Endpoint::port(Some(port_id.clone()), None);
                attach_endpoint(netlist, idx, endpoint);
                netlist.port_to_net.insert(port_id.clone(), idx);
            }
        }
    }
}

pub(crate) fn register_net(
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
    netlist.nets[idx].is_defined = true;
    let Some(name) = name else {
        return;
    };
    if netlist.nets[idx].name.is_none() {
        netlist.nets[idx].name = Some(name.to_string());
        netlist.by_name.insert(name.to_string(), idx);
    }
}

fn insert_new_net(
    netlist: &mut Netlist,
    net_id: &str,
    name: Option<&str>,
    is_defined: bool,
) -> usize {
    let idx = netlist.nets.len();
    let node = NetNode {
        id: net_id.to_string(),
        name: name.map(str::to_string),
        endpoints: Vec::new(),
        trace_ids: Vec::new(),
        is_defined,
        reference_count: 0,
    };
    netlist.nets.push(node);
    netlist.endpoint_ids_per_net.push(HashSet::new());
    netlist.by_id.insert(net_id.to_string(), idx);
    if let Some(n) = name {
        netlist.by_name.insert(n.to_string(), idx);
    }
    idx
}

fn attach_endpoint(netlist: &mut Netlist, net_idx: usize, endpoint: Endpoint) {
    if let Some(id) = endpoint.id() {
        if netlist.endpoint_ids_per_net[net_idx].insert(id.to_string()) {
            netlist.nets[net_idx].endpoints.push(endpoint);
        }
    } else {
        netlist.nets[net_idx].endpoints.push(endpoint);
    }
}
