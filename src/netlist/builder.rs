//! AST lowering into the Netlist connectivity graph.

use std::collections::{HashMap, HashSet};

use super::endpoint::Endpoint;
use super::node::NetNode;
use super::{DanglingRef, Netlist};
use crate::parser::{Circuit, Pad, PcbPort, Port, Trace};

type PortMap<'a> = HashMap<&'a str, &'a Port>;

/// Lowers the AST in dependency order: declared nets, port nets, traces,
/// then pads, which resolve their net through the ports traces connected.
pub fn build_netlist(circuit: &Circuit) -> Netlist {
    let mut netlist = Netlist::default();
    let ports: PortMap<'_> = circuit
        .ports()
        .filter_map(|p| p.port_id.as_deref().map(|id| (id, p)))
        .collect();
    index_defined_nets(&mut netlist, circuit);
    index_ports(&mut netlist, circuit);
    for (index, trace) in circuit.traces().enumerate() {
        index_trace(&mut netlist, index, trace, &ports);
    }
    let pcb_ports: HashMap<&str, &PcbPort> = circuit
        .pcb_ports()
        .filter_map(|p| p.pcb_port_id.as_deref().map(|id| (id, p)))
        .collect();
    for (index, pad) in circuit.pads().enumerate() {
        index_pad(&mut netlist, index, pad, &ports, &pcb_ports);
    }
    netlist
}

fn index_defined_nets(netlist: &mut Netlist, circuit: &Circuit) {
    for net in circuit.nets() {
        if let Some(net_id) = net.net_id.as_deref() {
            register_net(netlist, net_id, net.name.as_deref(), true);
        }
    }
}

fn index_ports(netlist: &mut Netlist, circuit: &Circuit) {
    for port in circuit.ports() {
        let Some(net_id) = port.net_id.as_deref() else {
            continue;
        };
        let idx = register_net(netlist, net_id, None, false);
        netlist.nets[idx].reference_count += 1;
        attach_port(netlist, idx, port);
    }
}

/// `source_trace "<id>"`, or the trace's position when it has no id.
fn trace_label(trace: &Trace, index: usize) -> String {
    match trace.trace_id.as_deref() {
        Some(id) => format!("source_trace \"{id}\""),
        None => format!("source_trace #{index}"),
    }
}

fn index_trace(netlist: &mut Netlist, index: usize, trace: &Trace, ports: &PortMap<'_>) {
    let mut resolved = Vec::new();
    for port_id in &trace.connected_source_port_ids {
        match ports.get(port_id.as_str()) {
            Some(port) => resolved.push(*port),
            None => netlist.dangling_refs.push(DanglingRef {
                owner: trace_label(trace, index),
                target: format!("source_port \"{port_id}\""),
            }),
        }
    }

    let net_indices: Vec<usize> = if trace.connected_source_net_ids.is_empty() {
        port_only_net(netlist, index, trace, &resolved)
            .into_iter()
            .collect()
    } else {
        trace
            .connected_source_net_ids
            .iter()
            .map(|net_id| register_net(netlist, net_id, None, false))
            .collect()
    };

    for idx in net_indices {
        netlist.nets[idx].reference_count += 1;
        for port in &resolved {
            attach_port(netlist, idx, port);
        }
    }
}

/// The net a trace without `connected_source_net_ids` joins: the net one of
/// its ports already belongs to, or a synthetic net unique to this trace.
fn port_only_net(
    netlist: &mut Netlist,
    index: usize,
    trace: &Trace,
    resolved: &[&Port],
) -> Option<usize> {
    if resolved.is_empty() {
        return None;
    }
    let existing = resolved
        .iter()
        .filter_map(|p| p.port_id.as_deref())
        .find_map(|id| netlist.port_to_net.get(id).copied());
    Some(existing.unwrap_or_else(|| {
        let synthetic_id = trace
            .connectivity_key
            .clone()
            .or_else(|| trace.trace_id.clone())
            .unwrap_or_else(|| format!("source_trace#{index}"));
        register_net(netlist, &synthetic_id, trace.name.as_deref(), true)
    }))
}

/// `source_pad "<id>"`, or the pad's position when it has no id.
fn pad_label(pad: &Pad, index: usize) -> String {
    match pad.pad_id.as_deref() {
        Some(id) => format!("pad \"{id}\""),
        None => format!("pad #{index}"),
    }
}

/// The `source_port_id` a pad stands for, through its direct `port_id` or
/// the canonical `pcb_port_id` chain. Every broken link is recorded.
fn linked_port_id<'a>(
    netlist: &mut Netlist,
    index: usize,
    pad: &'a Pad,
    ports: &PortMap<'_>,
    pcb_ports: &HashMap<&str, &'a PcbPort>,
) -> Option<&'a str> {
    let (port_id, owner) = match (pad.port_id.as_deref(), pad.pcb_port_id.as_deref()) {
        (Some(port_id), _) => (port_id, pad_label(pad, index)),
        (None, Some(pcb_port_id)) => {
            let Some(pcb_port) = pcb_ports.get(pcb_port_id) else {
                netlist.dangling_refs.push(DanglingRef {
                    owner: pad_label(pad, index),
                    target: format!("pcb_port \"{pcb_port_id}\""),
                });
                return None;
            };
            let owner = format!("pcb_port \"{pcb_port_id}\"");
            let Some(port_id) = pcb_port.source_port_id.as_deref() else {
                netlist.dangling_refs.push(DanglingRef {
                    owner,
                    target: "a source_port (no source_port_id)".to_string(),
                });
                return None;
            };
            (port_id, owner)
        }
        (None, None) => return None,
    };
    if ports.contains_key(port_id) {
        return Some(port_id);
    }
    netlist.dangling_refs.push(DanglingRef {
        owner,
        target: format!("source_port \"{port_id}\""),
    });
    None
}

fn index_pad(
    netlist: &mut Netlist,
    index: usize,
    pad: &Pad,
    ports: &PortMap<'_>,
    pcb_ports: &HashMap<&str, &PcbPort>,
) {
    let port_id = linked_port_id(netlist, index, pad, ports, pcb_ports);
    let idx = match pad.net_id.as_deref() {
        Some(net_id) => {
            let idx = register_net(netlist, net_id, None, false);
            netlist.nets[idx].reference_count += 1;
            Some(idx)
        }
        None => port_id.and_then(|id| netlist.port_to_net.get(id).copied()),
    };
    let Some(idx) = idx else {
        return;
    };
    if let Some(pad_id) = pad.pad_id.as_deref() {
        netlist.pad_to_net.insert(pad_id.to_string(), idx);
    }
    if !port_id.is_some_and(|id| netlist.endpoint_ids_per_net[idx].contains(id)) {
        attach_endpoint(netlist, idx, Endpoint::pad(pad.pad_id.clone()));
    }
}

fn attach_port(netlist: &mut Netlist, idx: usize, port: &Port) {
    attach_endpoint(netlist, idx, Endpoint::port(port));
    if let Some(id) = port.port_id.as_deref() {
        netlist.port_to_net.insert(id.to_string(), idx);
    }
}

fn register_net(
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

fn attach_endpoint(netlist: &mut Netlist, net_idx: usize, endpoint: Endpoint) {
    let is_new = match endpoint.id.as_deref() {
        Some(id) => netlist.endpoint_ids_per_net[net_idx].insert(id.to_string()),
        None => true,
    };
    if is_new {
        netlist.nets[net_idx].endpoints.push(endpoint);
    }
}
