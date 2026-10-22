//! AST lowering into the Netlist connectivity graph.

use std::collections::HashMap;

use super::connectivity::{Connectivity, PortMap};
use super::groups::collect;
use super::pads::index_pad;
use super::registry::{attach_port, insert_net};
use super::Netlist;
use crate::parser::{Circuit, PcbPort, Port};

/// Lowers the AST in two stages: connectivity (a union-find over nets,
/// ports and traces), then one net per connected component. Pads come last,
/// since they resolve their net through the ports traces connected.
pub fn build_netlist(circuit: &Circuit) -> Netlist {
    let mut netlist = Netlist::default();
    let ports: PortMap<'_> = circuit
        .ports()
        .filter_map(|p| p.port_id.as_deref().map(|id| (id, p)))
        .collect();
    let (mut graph, joined) = Connectivity::from_circuit(circuit, &ports);
    let mut root_to_net = HashMap::new();
    for group in collect(&mut graph) {
        let idx = insert_net(&mut netlist, group.node(&graph));
        for id in &group.net_ids {
            netlist.by_id.insert(id.clone(), idx);
        }
        netlist.ghosts.extend(group.ghosts(&graph));
        netlist.shorts.extend(group.short(&graph));
        root_to_net.insert(group.root, idx);
    }
    let order = attach_order(circuit, joined, &ports);
    attach_ports(&mut netlist, &mut graph, order, &root_to_net);
    netlist.dangling_refs = std::mem::take(&mut graph.dangling);
    index_pads(&mut netlist, circuit, &ports);
    netlist
}

fn index_pads(netlist: &mut Netlist, circuit: &Circuit, ports: &PortMap<'_>) {
    let pcb_ports: HashMap<&str, &PcbPort> = circuit
        .pcb_ports()
        .filter_map(|p| p.pcb_port_id.as_deref().map(|id| (id, p)))
        .collect();
    for (index, pad) in circuit.pads().enumerate() {
        index_pad(netlist, index, pad, ports, &pcb_ports);
    }
}

type Attachment<'a> = (&'a Port, Option<usize>);

/// Ports attach in the order the old lowering used: ports carrying a
/// `net_id` first, then each trace's ports, so endpoint order is stable.
fn attach_order<'a>(
    circuit: &'a Circuit,
    joined: Vec<Option<usize>>,
    ports: &PortMap<'a>,
) -> Vec<Attachment<'a>> {
    let on_nets = circuit
        .ports()
        .zip(joined)
        .filter(|(port, _)| port.net_id.is_some());
    let on_traces = circuit
        .traces()
        .flat_map(|trace| &trace.connected_source_port_ids)
        .filter_map(|id| ports.get(id.as_str()).map(|port| (*port, None)));
    on_nets.chain(on_traces).collect()
}

fn attach_ports(
    netlist: &mut Netlist,
    graph: &mut Connectivity,
    order: Vec<Attachment<'_>>,
    root_to_net: &HashMap<usize, usize>,
) {
    for (port, joined) in order {
        if let Some(node) = graph.port_node(port, joined) {
            let root = graph.sets.find(node);
            attach_port(netlist, root_to_net[&root], port);
        }
    }
}
