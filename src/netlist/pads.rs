//! Pad lowering: resolving each pad to the port and net it stands for.

use std::collections::HashMap;

use super::connectivity::PortMap;
use super::endpoint::Endpoint;
use super::registry::attach_endpoint;
use super::{DanglingRef, Netlist};
use crate::parser::{Pad, PcbPort};

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

pub(super) fn index_pad(
    netlist: &mut Netlist,
    index: usize,
    pad: &Pad,
    ports: &PortMap<'_>,
    pcb_ports: &HashMap<&str, &PcbPort>,
) {
    let port_id = linked_port_id(netlist, index, pad, ports, pcb_ports);
    let idx = match pad.net_id.as_deref() {
        Some(net_id) => netlist.by_id.get(net_id).copied(),
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
