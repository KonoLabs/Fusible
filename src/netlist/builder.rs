//! AST lowering into the Netlist connectivity graph.

use std::collections::HashMap;

use super::pads::index_pad;
use super::registry::{attach_port, register_net};
use super::{DanglingRef, Netlist};
use crate::parser::{Circuit, PcbPort, Port, Trace};

pub(super) type PortMap<'a> = HashMap<&'a str, &'a Port>;

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
