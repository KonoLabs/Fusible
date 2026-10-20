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
    pub pin_number: Option<u32>,
    pub hints: Vec<String>,
}

impl Endpoint {
    /// Constructs a port endpoint.
    pub fn port(id: Option<String>, name: Option<String>) -> Self {
        Self::with_details(id, None, name, None, Vec::new())
    }

    /// Constructs a port endpoint with full component, pin number, and hint details.
    pub fn with_details(
        id: Option<String>,
        component_id: Option<String>,
        name: Option<String>,
        pin_number: Option<u32>,
        hints: Vec<String>,
    ) -> Self {
        let comp =
            component_id.or_else(|| id.as_deref().and_then(parse_comp_id).map(str::to_string));
        Self {
            id,
            kind: EndpointKind::Port,
            component_id: comp,
            name,
            pin_number,
            hints,
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
            pin_number: None,
            hints: Vec::new(),
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

    /// Pin number accessor if available.
    pub fn pin_number(&self) -> Option<u32> {
        self.pin_number
    }

    /// Port hints accessor.
    pub fn hints(&self) -> &[String] {
        &self.hints
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
