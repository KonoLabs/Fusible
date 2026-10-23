//! Unions that can short declared nets. Every join goes through
//! [`Connectivity::link`], which records the element that first tied two
//! components holding declared nets together.

use std::collections::HashMap;

use super::connectivity::{Connectivity, PortMap, TraceJoin};
use super::pads::pad_label;
use crate::parser::{Circuit, Pad, PcbPort};

impl Connectivity {
    /// Unions `a` and `b`, recording `label` when both sides already hold a declared net.
    pub(super) fn link(&mut self, a: usize, b: usize, label: &str) {
        let (ra, rb) = (self.sets.find(a), self.sets.find(b));
        if ra != rb && self.holds_declared(ra) && self.holds_declared(rb) {
            self.short_joins.push(TraceJoin {
                label: label.to_string(),
                anchor: a,
            });
        }
        self.sets.union(a, b);
    }

    fn holds_declared(&mut self, root: usize) -> bool {
        for i in 0..self.declared_nodes.len() {
            if self.sets.find(self.declared_nodes[i]) == root {
                return true;
            }
        }
        false
    }

    /// Ties each pad's `net_id` to the port the pad is linked to, so a pad on
    /// GND whose port sits on 3V3 is a short, like a trace joining them.
    pub(super) fn join_pads(&mut self, circuit: &Circuit, ports: &PortMap<'_>) {
        let pcb_ports: HashMap<&str, &PcbPort> = circuit
            .pcb_ports()
            .filter_map(|p| p.pcb_port_id.as_deref().map(|id| (id, p)))
            .collect();
        for (index, pad) in circuit.pads().enumerate() {
            let Some(net_id) = pad.net_id.as_deref() else {
                continue;
            };
            let net = self.net(net_id);
            self.reference(net_id);
            let linked = linked_port(pad, &pcb_ports).filter(|id| ports.contains_key(id));
            if let Some(port_id) = linked {
                let port = self.port(Some(port_id));
                self.link(port, net, &pad_label(pad, index));
            }
        }
    }
}

fn linked_port<'a>(pad: &'a Pad, pcb_ports: &HashMap<&str, &'a PcbPort>) -> Option<&'a str> {
    pad.port_id.as_deref().or_else(|| {
        pcb_ports
            .get(pad.pcb_port_id.as_deref()?)?
            .source_port_id
            .as_deref()
    })
}
