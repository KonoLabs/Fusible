//! The rule set: every electrical/structural check the fuse can blow on.
//!
//! Phase 1 ships three rules that together make the pipeline **fail-closed**:
//!
//! * `UnconnectedNetRule` — a net with fewer than two endpoints is floating;
//! * `MalformedElementRule` — a recognized element without its identifier
//!   cannot be verified, so it is a violation, not a shrug;
//! * `GhostNetRule` — a `net_id` referenced by a port or pad but defined by
//!   no `source_net` describes copper that does not exist.
//!
//! The division of labor matters: `UnconnectedNetRule` skips nets it cannot
//! identify, *because* `MalformedElementRule` reports them. A rule may only
//! stay silent when another registered rule speaks for what it skipped.
//!
//! One rule per file — filet's functions-per-file budget counts `impl`
//! methods, so a hub module keeps every rule comfortably under the limit.

mod bypass_cap;
mod dangling_reference;
mod ghost_net;
mod malformed_element;
mod pin_mux;
mod schematic_scale;
#[cfg(test)]
mod tests;
mod trace_clearance;
mod unconnected_net;

pub use bypass_cap::BypassCapRule;
pub use dangling_reference::DanglingReferenceRule;
pub use ghost_net::GhostNetRule;
pub use malformed_element::MalformedElementRule;
pub use pin_mux::PinMuxRule;
pub use schematic_scale::SchematicScaleRule;
pub use trace_clearance::TraceClearanceRule;
pub use unconnected_net::UnconnectedNetRule;
