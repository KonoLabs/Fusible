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
    let comp_map: std::collections::HashMap<_, _> = circuit
        .components()
        .filter_map(|c| {
            c.component_id
                .as_deref()
                .map(|id| (id, c.name.as_deref().unwrap_or(id)))
        })
        .collect();

    for port in circuit.ports() {
        let comp = port
            .source_component_id
            .as_deref()
            .and_then(|cid| comp_map.get(cid).copied())
            .map(str::to_string)
            .or_else(|| port.source_component_id.clone());

        if let Some(net_id) = port.net_id.as_deref() {
            let idx = register_net(netlist, net_id, None, false);
            netlist.nets[idx].reference_count += 1;
            attach_endpoint(
                netlist,
                idx,
                Endpoint::with_details(
                    port.port_id.clone(),
                    comp,
                    port.name.clone(),
                    port.pin_number,
                    port.port_hints.clone().unwrap_or_default(),
                ),
            );
            if let Some(id) = port.port_id.as_deref() {
                netlist.port_to_net.insert(id.to_string(), idx);
            }
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
    let comp_map: std::collections::HashMap<_, _> = circuit
        .components()
        .filter_map(|c| {
            c.component_id
                .as_deref()
                .map(|id| (id, c.name.as_deref().unwrap_or(id)))
        })
        .collect();

    let port_map: std::collections::HashMap<_, _> = circuit
        .ports()
        .filter_map(|p| p.port_id.as_deref().map(|id| (id, p)))
        .collect();

    for trace in circuit.traces() {
        if !trace.connected_source_net_ids.is_empty() {
            for net_id in &trace.connected_source_net_ids {
                let idx = register_net(netlist, net_id, None, false);
                netlist.nets[idx].reference_count += 1;

                if let Some(trace_id) = trace.trace_id.as_deref() {
                    netlist.nets[idx].trace_ids.push(trace_id.to_string());
                }

                attach_trace_ports(
                    netlist,
                    idx,
                    &trace.connected_source_port_ids,
                    &port_map,
                    &comp_map,
                );
            }
        } else if !trace.connected_source_port_ids.is_empty() {
            let existing_net_idx = trace
                .connected_source_port_ids
                .iter()
                .find_map(|pid| netlist.port_to_net.get(pid).copied());

            let idx = existing_net_idx.unwrap_or_else(|| {
                let synthetic_id = trace
                    .connectivity_key
                    .as_deref()
                    .or(trace.trace_id.as_deref())
                    .unwrap_or("trace_net");
                register_net(netlist, synthetic_id, trace.name.as_deref(), true)
            });

            netlist.nets[idx].reference_count += 1;
            if let Some(trace_id) = trace.trace_id.as_deref() {
                netlist.nets[idx].trace_ids.push(trace_id.to_string());
            }

            attach_trace_ports(
                netlist,
                idx,
                &trace.connected_source_port_ids,
                &port_map,
                &comp_map,
            );
        }
    }
}

fn attach_trace_ports(
    netlist: &mut Netlist,
    net_idx: usize,
    port_ids: &[String],
    port_map: &std::collections::HashMap<&str, &crate::parser::Port>,
    comp_map: &std::collections::HashMap<&str, &str>,
) {
    for port_id in port_ids {
        let port_opt = port_map.get(port_id.as_str()).copied();
        let comp = port_opt
            .and_then(|p| p.source_component_id.as_deref())
            .and_then(|cid| comp_map.get(cid).copied())
            .map(str::to_string)
            .or_else(|| port_opt.and_then(|p| p.source_component_id.clone()));

        let endpoint = if let Some(p) = port_opt {
            Endpoint::with_details(
                Some(port_id.clone()),
                comp,
                p.name.clone(),
                p.pin_number,
                p.port_hints.clone().unwrap_or_default(),
            )
        } else {
            Endpoint::port(Some(port_id.clone()), None)
        };
        attach_endpoint(netlist, net_idx, endpoint);
        netlist.port_to_net.insert(port_id.clone(), net_idx);
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
