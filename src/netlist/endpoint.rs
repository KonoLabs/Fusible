//! Electrical connection endpoints (ports and pads).

/// The kind of electrical endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointKind {
    /// A component port / terminal (e.g. `U1.1`).
    Port,
    /// A physical mount pad on the PCB.
    Pad,
}

/// A physical connection site on a net (port or pad).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Endpoint {
    pub id: Option<String>,
    pub kind: EndpointKind,
    pub component_id: Option<String>,
    pub name: Option<String>,
}

impl Endpoint {
    /// Constructs a port endpoint.
    pub fn port(id: Option<String>, name: Option<String>) -> Self {
        let component_id = id.as_deref().and_then(parse_comp_id).map(str::to_string);
        Self {
            id,
            kind: EndpointKind::Port,
            component_id,
            name,
        }
    }

    /// Constructs a pad endpoint.
    pub fn pad(id: Option<String>) -> Self {
        let component_id = id.as_deref().and_then(parse_comp_id).map(str::to_string);
        Self {
            id,
            kind: EndpointKind::Pad,
            component_id,
            name: None,
        }
    }

    /// Identifier accessor (`port_id` or `pad_id`).
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Parent component identifier accessor.
    pub fn component_id(&self) -> Option<&str> {
        self.component_id.as_deref()
    }

    /// Human-readable pin name accessor.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// True if this endpoint is a component port.
    pub fn is_port(&self) -> bool {
        self.kind == EndpointKind::Port
    }

    /// True if this endpoint is a PCB pad.
    pub fn is_pad(&self) -> bool {
        self.kind == EndpointKind::Pad
    }
}

fn parse_comp_id(id: &str) -> Option<&str> {
    id.split_once('.').map(|(comp, _)| comp)
}
