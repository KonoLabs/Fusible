//! Electrical net graph node.

use super::endpoint::Endpoint;

/// An equipotential electrical net node holding resolved endpoints and traces.
#[derive(Debug, Clone)]
pub struct NetNode {
    pub id: String,
    pub name: Option<String>,
    pub endpoints: Vec<Endpoint>,
    pub trace_ids: Vec<String>,
    pub is_defined: bool,
    pub reference_count: usize,
}

impl NetNode {
    /// Canonical net identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Human-readable net name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the human name if known, or ID as fallback.
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.id)
    }

    /// Slice of connected endpoints.
    pub fn endpoints(&self) -> &[Endpoint] {
        &self.endpoints
    }

    /// Number of physical connections (ports + pads).
    pub fn endpoint_count(&self) -> usize {
        self.endpoints.len()
    }

    /// True if this net has fewer than two endpoints (floating).
    pub fn is_floating(&self) -> bool {
        self.endpoints.len() < 2
    }

    /// True if referenced but not declared in `source_net` (ghost net).
    pub fn is_ghost(&self) -> bool {
        !self.is_defined
    }

    /// Total count of references to this net across ports, pads, and traces.
    pub fn reference_count(&self) -> usize {
        self.reference_count
    }
}
