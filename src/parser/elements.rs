//! Strongly typed structs behind each `CircuitElement` variant.

use serde::Deserialize;

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

/// One vertex of a polygon pad outline.
#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub struct PadPoint {
    pub x: f64,
    pub y: f64,
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
    /// Canonical `pcb_smtpad` link: `pcb_port_id` -> `pcb_port.source_port_id`.
    #[serde(default, alias = "pcbPortId")]
    pub pcb_port_id: Option<String>,
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub height: Option<f64>,
    /// Circle pads carry a radius instead of width and height.
    #[serde(default)]
    pub radius: Option<f64>,
    /// Polygon pads carry their outline instead of a center and size.
    #[serde(default)]
    pub points: Option<Vec<PadPoint>>,
    #[serde(default)]
    pub layer: Option<String>,
}

/// The PCB side of a port.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PcbPort {
    #[serde(alias = "pcbPortId")]
    pub pcb_port_id: Option<String>,
    /// The `source_port` this PCB port realizes.
    #[serde(default, alias = "sourcePortId")]
    pub source_port_id: Option<String>,
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
