//! Electrical connection endpoints (ports and pads).

use crate::parser::Port;

/// A physical connection site on a net (port or pad).
///
/// `component_id` is the parent's `source_component_id`, never its display
/// name: rules key component metadata by id and resolve names only to print.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Endpoint {
    pub id: Option<String>,
    pub component_id: Option<String>,
    pub name: Option<String>,
}

impl Endpoint {
    /// Constructs a port endpoint from its `source_port`. Without a
    /// `source_component_id`, the `U1` prefix of a `U1.1` port id stands in.
    pub fn port(port: &Port) -> Self {
        let component_id = port
            .source_component_id
            .clone()
            .or_else(|| prefix_component_id(port.port_id.as_deref()));
        Self {
            id: port.port_id.clone(),
            component_id,
            name: port.name.clone(),
        }
    }

    /// Constructs a pad endpoint.
    pub fn pad(id: Option<String>) -> Self {
        Self {
            component_id: prefix_component_id(id.as_deref()),
            id,
            name: None,
        }
    }
}

fn prefix_component_id(id: Option<&str>) -> Option<String> {
    id?.split_once('.').map(|(comp, _)| comp.to_string())
}
