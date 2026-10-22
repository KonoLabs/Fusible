//! Deserialized Circuit JSON PCB elements and the model collecting them.

use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub struct PcbPoint {
    pub x: f64,
    pub y: f64,
}

fn missing() -> f64 {
    f64::NAN
}

#[derive(Debug, Deserialize, Default)]
pub struct PcbBoardElm {
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub center: Option<PcbPoint>,
    pub outline: Option<Vec<PcbPoint>>,
    pub thickness: Option<f64>,
    pub num_layers: Option<u8>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PcbHoleElm {
    pub x: f64,
    pub y: f64,
    pub hole_shape: Option<String>,
    pub hole_diameter: Option<f64>,
    pub hole_width: Option<f64>,
    pub hole_height: Option<f64>,
    pub ccw_rotation: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PcbSmtPadElm {
    #[serde(default = "missing")]
    pub x: f64,
    #[serde(default = "missing")]
    pub y: f64,
    pub points: Option<Vec<PcbPoint>>,
    pub rect_border_radius: Option<f64>,
    pub corner_radius: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub radius: Option<f64>,
    pub ccw_rotation: Option<f64>,
    pub layer: Option<String>,
    pub shape: Option<String>,
    pub soldermask_margin: Option<f64>,
    pub is_covered_with_solder_mask: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PcbTracePoint {
    pub route_type: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: Option<f64>,
    pub layer: Option<String>,
    pub hole_diameter: Option<f64>,
    pub outer_diameter: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PcbTraceElm {
    #[serde(default)]
    pub route: Vec<PcbTracePoint>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PcbPlatedHoleElm {
    pub x: f64,
    pub y: f64,
    pub shape: Option<String>,
    pub hole_diameter: Option<f64>,
    pub hole_width: Option<f64>,
    pub hole_height: Option<f64>,
    pub hole_offset_x: Option<f64>,
    pub hole_offset_y: Option<f64>,
    pub outer_diameter: Option<f64>,
    pub outer_width: Option<f64>,
    pub outer_height: Option<f64>,
    pub rect_pad_width: Option<f64>,
    pub rect_pad_height: Option<f64>,
    pub ccw_rotation: Option<f64>,
    pub hole_ccw_rotation: Option<f64>,
    pub rect_ccw_rotation: Option<f64>,
    pub is_covered_with_solder_mask: Option<bool>,
    pub soldermask_margin: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PcbViaElm {
    pub x: f64,
    pub y: f64,
    pub hole_diameter: Option<f64>,
    pub outer_diameter: Option<f64>,
    pub is_tented: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PcbSilkscreenPathElm {
    pub layer: Option<String>,
    pub stroke_width: Option<f64>,
    #[serde(default)]
    pub route: Vec<PcbPoint>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PcbSilkscreenTextElm {
    pub text: Option<String>,
    pub anchor_position: Option<PcbPoint>,
    pub center: Option<PcbPoint>,
    pub font_size: Option<f64>,
    pub layer: Option<String>,
    pub anchor_alignment: Option<String>,
    pub ccw_rotation: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PcbComponentElm {
    pub pcb_component_id: Option<String>,
    pub source_component_id: Option<String>,
    pub center: Option<PcbPoint>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub layer: Option<String>,
}

/// Generic container collecting all parsed PCB elements.
#[derive(Debug, Default)]
pub struct PcbModel {
    pub boards: Vec<PcbBoardElm>,
    pub smt_pads: Vec<PcbSmtPadElm>,
    pub traces: Vec<PcbTraceElm>,
    pub plated_holes: Vec<PcbPlatedHoleElm>,
    pub vias: Vec<PcbViaElm>,
    pub silkscreen_paths: Vec<PcbSilkscreenPathElm>,
    pub silkscreen_texts: Vec<PcbSilkscreenTextElm>,
    pub components: Vec<PcbComponentElm>,
    pub holes: Vec<PcbHoleElm>,
}
