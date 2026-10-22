//! Cutting nets out of the connected components.

use std::collections::{BTreeSet, HashMap};

use super::connectivity::Connectivity;
use super::node::NetNode;
use super::Short;

pub(super) struct Group {
    pub root: usize,
    pub net_ids: Vec<String>,
    pub first_trace: Option<String>,
}

pub(super) fn collect(graph: &mut Connectivity) -> Vec<Group> {
    let mut index: HashMap<usize, usize> = HashMap::new();
    let mut groups: Vec<Group> = Vec::new();
    for (node, id) in graph.net_ids.clone() {
        let root = graph.sets.find(node);
        let slot = *index.entry(root).or_insert_with(|| {
            groups.push(Group {
                root,
                net_ids: Vec::new(),
                first_trace: None,
            });
            groups.len() - 1
        });
        groups[slot].net_ids.push(id);
    }
    for trace in &graph.traces {
        let root = graph.sets.find(trace.anchor);
        let group = &mut groups[index[&root]];
        group.first_trace.get_or_insert_with(|| trace.label.clone());
    }
    groups
}

impl Group {
    fn is_declared(&self, graph: &Connectivity, id: &str) -> bool {
        graph.declared.contains_key(id)
    }

    fn is_synthetic(&self, graph: &Connectivity, id: &str) -> bool {
        !self.is_declared(graph, id) && graph.synthetic.contains_key(id)
    }

    fn pick_id<'a>(&'a self, graph: &Connectivity) -> &'a str {
        let ranked = |declared: bool, synthetic: bool| {
            let mut matching = self.net_ids.iter().filter(|id| {
                self.is_declared(graph, id) == declared && self.is_synthetic(graph, id) == synthetic
            });
            if synthetic {
                matching.next()
            } else {
                matching.min()
            }
        };
        ranked(true, false)
            .or_else(|| ranked(false, false))
            .or_else(|| ranked(false, true))
            .or(self.net_ids.first())
            .map(String::as_str)
            .unwrap_or_default()
    }

    pub(super) fn node(&self, graph: &Connectivity) -> NetNode {
        let id = self.pick_id(graph);
        let name = match graph.declared.get(id) {
            Some(name) => name.clone(),
            None => graph.synthetic.get(id).cloned().flatten(),
        };
        let has_real_net = self.net_ids.iter().any(|n| !self.is_synthetic(graph, n));
        let has_declared = self.net_ids.iter().any(|n| self.is_declared(graph, n));
        NetNode {
            id: id.to_string(),
            name,
            endpoints: Vec::new(),
            is_defined: has_declared || !has_real_net,
            reference_count: self.reference_count(graph),
        }
    }

    fn reference_count(&self, graph: &Connectivity) -> usize {
        self.net_ids
            .iter()
            .map(|id| graph.refs.get(id).copied().unwrap_or(0))
            .sum()
    }

    pub(super) fn ghosts(&self, graph: &Connectivity) -> Vec<NetNode> {
        self.net_ids
            .iter()
            .filter(|id| !self.is_declared(graph, id) && !self.is_synthetic(graph, id))
            .map(|id| NetNode {
                id: id.clone(),
                name: None,
                endpoints: Vec::new(),
                is_defined: false,
                reference_count: graph.refs.get(id).copied().unwrap_or(0),
            })
            .collect()
    }

    pub(super) fn short(&self, graph: &Connectivity) -> Option<Short> {
        let names: BTreeSet<String> = self
            .net_ids
            .iter()
            .filter_map(|id| graph.declared.get(id).map(|name| (id, name)))
            .map(|(id, name)| name.clone().unwrap_or_else(|| id.clone()))
            .collect();
        (names.len() > 1).then(|| Short {
            nets: names.into_iter().collect(),
            trace: self
                .first_trace
                .clone()
                .unwrap_or_else(|| "a source_port net_id".to_string()),
        })
    }
}
