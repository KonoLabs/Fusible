//! Step 2 — Circuit JSON parser (partial model).
//!
//! # Architecture choice: Serde and heterogeneous JSON
//!
//! tscircuit's Circuit JSON is an **array of heterogeneous objects**: every
//! element carries a discriminating `"type"` field (`source_component`,
//! `source_net`, `source_port`, `source_pad`, `source_trace`, ...).
//!
//! ```json
//! [
//!   { "type": "source_component", ... },
//!   { "type": "source_net", ... }
//! ]
//! ```
//!
//! This is the canonical *internally tagged enum* format for Serde:
//!
//! * `#[serde(tag = "type")]` names the discriminator: Serde reads `type`
//!   first, then delegates deserialization of the **rest** of the object to
//!   the matching variant. No hand-rolled "read `type`, `match`, then
//!   `from_value`" — Serde does it in one pass.
//! * Variants are **newtype variants** (`Component(Component)`): each one
//!   wraps its strongly typed struct. Serde accepts a struct behind an
//!   internally tagged variant (its fields are read as if flattened into the
//!   original object).
//! * `#[serde(rename = "...")]` maps the JSON literal to a readable Rust
//!   name (`Component`, `Net`, `Pad`, ...). The JSON tag rarely moves; the
//!   Rust name has to stay legible everywhere else in the code.
//! * `#[serde(other)]` on the `Unknown` variant: element types Phase 1 does
//!   not model (`silkscreen_text`, `footprint`, `schematic_box`, ...) are
//!   **skipped instead of failing the whole parse**. This is the central
//!   robustness choice: the linter must complain about electrical rules, not
//!   about our model being partial. (Serde constraint: `other` only exists
//!   for internally/adjacent tagged enums, on a unit variant — exactly our
//!   case. A `rename` on `Unknown` would only affect serialization, which
//!   never happens here.)
//! * Serde silently ignores unknown fields **inside** a variant: the partial
//!   model can grow (geometry, phase-2 pin-muxing, ...) without breaking
//!   existing files.
//!
//! # Field names: snake_case is the source of truth
//!
//! circuit-json is snake_case, enforced by tscircuit's own
//! `check-snake-case` CI. Every identifier field therefore accepts the
//! canonical snake_case key (`source_component_id`, `source_net_id`,
//! `connected_source_port_ids`, ...), and the camelCase aliases
//! (`sourceComponentId`, `connectedSourcePortIds`) are kept purely as
//! defensive tolerance for older files or alternative generators — they cost
//! nothing. Passing *both* spellings for one field is a hard
//! `duplicate field` error: ambiguous input must not parse quietly.
//!
//! # `None` never means "not applicable"
//!
//! Every identifier field is `Option`, because the model is deliberately
//! *partial* at the deserialization level. But canonical circuit-json always
//! carries these identifiers, so `None` marks input that deviates from the
//! format — and a linter that silently skips what it cannot read would
//! validate malformed input. The linter's `malformed_element` rule surfaces
//! exactly that case, which is what keeps the whole pipeline fail-closed.
//!
//! The parse diagnostic (rustc-style label on the Serde position) lives in
//! the `diagnostic` submodule: only the data model stays here.
//!
// The model is deliberately *partial*: Pad/Trace fields and a few others are
// only consumed in phase 2 (pin-muxing, codegen). We neither dead-code them
// away nor delete them early.
#![allow(dead_code)]

mod diagnostic;
#[cfg(test)]
mod tests;

pub use diagnostic::JsonDiagnostic;

use std::path::Path;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Internally tagged enum: the parsing root
// ---------------------------------------------------------------------------

/// One element of the Circuit JSON array.
///
/// Each variant maps to one possible value of the `"type"` field.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum CircuitElement {
    /// A component placed on the board (resistor, MCU, capacitor, ...).
    #[serde(rename = "source_component")]
    Component(Component),
    /// A logical connection (the "net": GND, +3V3, I2C_SDA, ...).
    #[serde(rename = "source_net")]
    Net(Net),
    /// A pin / port of a component.
    #[serde(rename = "source_port")]
    Port(Port),
    /// A mount pad (physical solder surface on the PCB).
    #[serde(rename = "source_pad", alias = "pcb_smtpad")]
    Pad(Pad),
    /// A copper trace joining two ports.
    #[serde(rename = "source_trace")]
    Trace(Trace),
    /// Type not modeled in phase 1: ignored rather than rejected.
    #[serde(other)]
    Unknown,
}

// ---------------------------------------------------------------------------
// Strongly typed structs
// ---------------------------------------------------------------------------

/// A component.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Component {
    /// Stable identifier, e.g. `U1` or `source_component_0`.
    #[serde(alias = "source_component_id", alias = "sourceComponentId")]
    pub component_id: Option<String>,
    /// Component kind: `chip`, `resistor`, `capacitor`, ...
    #[serde(alias = "ftype")]
    pub kind: Option<String>,
    /// Human-readable designator, e.g. `U1` or `ESP32`.
    #[serde(default)]
    pub name: Option<String>,
}

/// A net: the logical connection between pins.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Net {
    #[serde(alias = "source_net_id", alias = "sourceNetId", alias = "netId")]
    pub net_id: Option<String>,
    /// Human-readable name shown in diagnostics, e.g. `GND`.
    #[serde(alias = "netName")]
    pub name: Option<String>,
}

/// A pin / port of a component.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Port {
    #[serde(alias = "source_port_id", alias = "sourcePortId")]
    pub port_id: Option<String>,
    #[serde(alias = "portName")]
    pub name: Option<String>,
    /// The net attached to this port — the datum `UnconnectedNetRule`
    /// counts. Ports legitimately without a net stay `None` (an unconnected
    /// pin is fine for this field; a missing `port_id` is not, see the
    /// `malformed_element` rule).
    #[serde(alias = "source_net_id", alias = "sourceNetId", alias = "netId")]
    pub net_id: Option<String>,
    /// Parent component identifier, e.g. `source_component_0`.
    #[serde(default, alias = "source_component_id", alias = "sourceComponentId")]
    pub source_component_id: Option<String>,
    /// Pin number on footprint or package.
    #[serde(default, alias = "pin_number", alias = "pinNumber")]
    pub pin_number: Option<u32>,
    /// Alternative pin hints or labels.
    #[serde(default, alias = "port_hints", alias = "portHints")]
    pub port_hints: Option<Vec<String>>,
}

/// A mount pad (the physical analogue of a port).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Pad {
    #[serde(
        alias = "source_pad_id",
        alias = "sourcePadId",
        alias = "pcb_smtpad_id"
    )]
    pub pad_id: Option<String>,
    #[serde(alias = "source_net_id", alias = "sourceNetId", alias = "netId")]
    pub net_id: Option<String>,
    #[serde(default, alias = "source_port_id", alias = "port_id")]
    pub port_id: Option<String>,
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub height: Option<f64>,
    #[serde(default)]
    pub layer: Option<String>,
}

/// A copper trace.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Trace {
    #[serde(alias = "source_trace_id", alias = "sourceTraceId")]
    pub trace_id: Option<String>,
    /// Ports joined by this trace. The canonical circuit-json field is
    /// `connected_source_port_ids` (snake_case, required by the schema);
    /// the camelCase spelling is kept as a defensive alias.
    #[serde(
        default,
        alias = "connectedSourcePortIds",
        alias = "connected_port_ids",
        alias = "connectedPortIds"
    )]
    pub connected_source_port_ids: Vec<String>,
    /// Nets joined by this trace.
    #[serde(
        default,
        alias = "connectedSourceNetIds",
        alias = "connected_net_ids",
        alias = "connectedNetIds"
    )]
    pub connected_source_net_ids: Vec<String>,
    /// Optional display or trace name.
    #[serde(default, alias = "display_name", alias = "displayName")]
    pub name: Option<String>,
    /// Optional subcircuit connectivity map key.
    #[serde(default, alias = "subcircuit_connectivity_map_key")]
    pub connectivity_key: Option<String>,
}

// ---------------------------------------------------------------------------
// AST: a typed view over the whole document
// ---------------------------------------------------------------------------

/// The circuit AST: the raw element array plus typed accessors.
///
/// Instead of `match`ing over `elements` everywhere, the per-type *views*
/// live here. The linter — and later the `mcu::` module — consume these
/// iterators without knowing anything about the JSON shape.
#[derive(Debug, Clone)]
pub struct Circuit {
    pub elements: Vec<CircuitElement>,
}

impl Circuit {
    pub fn components(&self) -> impl Iterator<Item = &Component> {
        self.elements.iter().filter_map(|e| match e {
            CircuitElement::Component(c) => Some(c),
            _ => None,
        })
    }

    pub fn nets(&self) -> impl Iterator<Item = &Net> {
        self.elements.iter().filter_map(|e| match e {
            CircuitElement::Net(n) => Some(n),
            _ => None,
        })
    }

    pub fn ports(&self) -> impl Iterator<Item = &Port> {
        self.elements.iter().filter_map(|e| match e {
            CircuitElement::Port(p) => Some(p),
            _ => None,
        })
    }

    pub fn pads(&self) -> impl Iterator<Item = &Pad> {
        self.elements.iter().filter_map(|e| match e {
            CircuitElement::Pad(p) => Some(p),
            _ => None,
        })
    }

    pub fn traces(&self) -> impl Iterator<Item = &Trace> {
        self.elements.iter().filter_map(|e| match e {
            CircuitElement::Trace(t) => Some(t),
            _ => None,
        })
    }
}

// ---------------------------------------------------------------------------
// Parsing entry point
// ---------------------------------------------------------------------------

/// Deserializes the Circuit JSON array.
///
/// `path` only names the file inside the diagnostic.
pub fn parse(source: &str, path: &Path) -> Result<Circuit, JsonDiagnostic> {
    let elements: Vec<CircuitElement> =
        serde_json::from_str(source).map_err(|err| JsonDiagnostic::new(err, source, path))?;
    Ok(Circuit { elements })
}
