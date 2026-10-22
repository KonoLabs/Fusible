//! Union-find over source ids, built from `source_trace` connectivity.

use std::collections::BTreeMap;

use serde_json::Value;

use super::json::type_of;

/// Union-find over `source_port` and `source_net` ids, built from `source_trace` connectivity.
#[derive(Default)]
pub(super) struct Connectivity {
    parent: BTreeMap<String, String>,
}

impl Connectivity {
    pub(super) fn from_elements(elements: &[Value]) -> Self {
        let mut conn = Self::default();
        for el in elements.iter().filter(|el| type_of(el) == "source_trace") {
            let ids: Vec<String> = ["connected_source_port_ids", "connected_source_net_ids"]
                .iter()
                .filter_map(|key| el.get(*key).and_then(Value::as_array))
                .flatten()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect();
            for pair in ids.windows(2) {
                conn.union(&pair[0], &pair[1]);
            }
            if let [only] = ids.as_slice() {
                conn.find(only);
            }
        }
        conn
    }

    fn find(&mut self, id: &str) -> String {
        let mut root = id.to_string();
        while let Some(parent) = self.parent.get(&root).filter(|p| **p != root) {
            root = parent.clone();
        }
        let mut cur = id.to_string();
        while cur != root {
            let next = self
                .parent
                .insert(cur, root.clone())
                .unwrap_or_else(|| root.clone());
            cur = next;
        }
        self.parent
            .entry(root.clone())
            .or_insert_with(|| root.clone());
        root
    }

    fn union(&mut self, a: &str, b: &str) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            let (keep, drop) = if ra < rb { (ra, rb) } else { (rb, ra) };
            self.parent.insert(drop, keep);
        }
    }

    pub(super) fn net_of(&mut self, id: &str) -> Option<String> {
        self.parent.contains_key(id).then(|| self.find(id))
    }
}
