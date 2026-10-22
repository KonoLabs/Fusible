//! Connectivity first: every net, port and trace joins one disjoint-set
//! forest, and nets are cut from its components afterwards.

use std::collections::HashMap;

use super::labels::{synthetic_id, trace_label};
use super::union_find::UnionFind;
use super::DanglingRef;
use crate::parser::{Circuit, Port, Trace};

pub(super) type PortMap<'a> = HashMap<&'a str, &'a Port>;

pub(super) struct TraceJoin {
    pub label: String,
    pub anchor: usize,
}

#[derive(Default)]
pub(super) struct Connectivity {
    pub sets: UnionFind,
    pub net_ids: Vec<(usize, String)>,
    net_nodes: HashMap<String, usize>,
    port_nodes: HashMap<String, usize>,
    pub declared: HashMap<String, Option<String>>,
    pub synthetic: HashMap<String, Option<String>>,
    pub refs: HashMap<String, usize>,
    pub traces: Vec<TraceJoin>,
    pub dangling: Vec<DanglingRef>,
}

impl Connectivity {
    pub(super) fn from_circuit(
        circuit: &Circuit,
        ports: &PortMap<'_>,
    ) -> (Self, Vec<Option<usize>>) {
        let mut graph = Self::default();
        for net in circuit.nets() {
            if let Some(id) = net.net_id.as_deref() {
                graph.net(id);
                graph.declared.insert(id.to_string(), net.name.clone());
            }
        }
        let anonymous = circuit.ports().map(|p| graph.join_port(p)).collect();
        for (index, trace) in circuit.traces().enumerate() {
            graph.join_trace(index, trace, ports);
        }
        for net_id in circuit.pads().filter_map(|pad| pad.net_id.as_deref()) {
            graph.net(net_id);
            graph.reference(net_id);
        }
        (graph, anonymous)
    }

    pub(super) fn port_node(&self, port: &Port, joined: Option<usize>) -> Option<usize> {
        joined.or_else(|| self.port_nodes.get(port.port_id.as_deref()?).copied())
    }

    fn net(&mut self, id: &str) -> usize {
        if let Some(&node) = self.net_nodes.get(id) {
            return node;
        }
        let node = self.sets.add();
        self.net_nodes.insert(id.to_string(), node);
        self.net_ids.push((node, id.to_string()));
        node
    }

    fn port(&mut self, id: Option<&str>) -> usize {
        let Some(id) = id else {
            return self.sets.add();
        };
        if let Some(&node) = self.port_nodes.get(id) {
            return node;
        }
        let node = self.sets.add();
        self.port_nodes.insert(id.to_string(), node);
        node
    }

    fn reference(&mut self, net_id: &str) {
        *self.refs.entry(net_id.to_string()).or_default() += 1;
    }

    fn join_port(&mut self, port: &Port) -> Option<usize> {
        let net_id = port.net_id.as_deref()?;
        let node = self.port(port.port_id.as_deref());
        let net = self.net(net_id);
        self.sets.union(node, net);
        self.reference(net_id);
        Some(node)
    }

    fn trace_ports(&mut self, label: &str, trace: &Trace, ports: &PortMap<'_>) -> Vec<usize> {
        let mut nodes = Vec::new();
        for port_id in &trace.connected_source_port_ids {
            if ports.contains_key(port_id.as_str()) {
                nodes.push(self.port(Some(port_id)));
            } else {
                self.dangling.push(DanglingRef {
                    owner: label.to_string(),
                    target: format!("source_port \"{port_id}\""),
                });
            }
        }
        nodes
    }

    fn join_trace(&mut self, index: usize, trace: &Trace, ports: &PortMap<'_>) {
        let label = trace_label(trace, index);
        let mut nodes = self.trace_ports(&label, trace, ports);
        let port_only = trace.connected_source_net_ids.is_empty() && !nodes.is_empty();
        for net_id in &trace.connected_source_net_ids {
            nodes.push(self.net(net_id));
            self.reference(net_id);
        }
        if port_only {
            let id = synthetic_id(trace, index);
            nodes.push(self.net(&id));
            self.reference(&id);
            self.synthetic
                .entry(id)
                .or_insert_with(|| trace.name.clone());
        }
        let Some(&anchor) = nodes.first() else {
            return;
        };
        for &node in &nodes {
            self.sets.union(anchor, node);
        }
        self.traces.push(TraceJoin { label, anchor });
    }
}
