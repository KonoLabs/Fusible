//! Electrical net graph node.

use super::endpoint::Endpoint;

/// An equipotential electrical net node holding its resolved endpoints.
#[derive(Debug, Clone)]
pub struct NetNode {
    pub id: String,
    pub name: Option<String>,
    pub endpoints: Vec<Endpoint>,
    pub is_defined: bool,
    pub reference_count: usize,
}

impl NetNode {
    /// Returns the human name if known, or ID as fallback.
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.id)
    }

    /// True if this net has fewer than two endpoints (floating).
    pub fn is_floating(&self) -> bool {
        self.endpoints.len() < 2
    }
}
