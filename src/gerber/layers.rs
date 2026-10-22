//! Layer builders mapping Circuit JSON elements to RS-274X and Excellon files.

use serde::Deserialize;

use crate::gerber::apertures::{Aperture, ApertureDictionary};
use crate::gerber::coordinates::Coord;
use crate::gerber::excellon::ExcellonWriter;
use crate::gerber::writer::GerberWriter;

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub struct PcbPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Deserialize, Default)]
pub struct PcbBoardElm {
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub center: Option<PcbPoint>,
    pub outline: Option<Vec<PcbPoint>>,
    pub thickness: Option<f64>,
}

impl PcbBoardElm {
    /// Returns the closed board outline, deriving a rectangle from `width`, `height`, and `center` when no explicit outline is set.
    pub fn outline_points(&self) -> Vec<PcbPoint> {
        if let Some(outline) = &self.outline {
            return outline.clone();
        }
        let w = self.width.unwrap_or(80.0) / 2.0;
        let h = self.height.unwrap_or(60.0) / 2.0;
        let cx = self.center.as_ref().map(|c| c.x).unwrap_or(0.0);
        let cy = self.center.as_ref().map(|c| c.y).unwrap_or(0.0);
        vec![
            PcbPoint {
                x: cx - w,
                y: cy - h,
            },
            PcbPoint {
                x: cx + w,
                y: cy - h,
            },
            PcbPoint {
                x: cx + w,
                y: cy + h,
            },
            PcbPoint {
                x: cx - w,
                y: cy + h,
            },
            PcbPoint {
                x: cx - w,
                y: cy - h,
            },
        ]
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct PcbHoleElm {
    pub x: f64,
    pub y: f64,
    pub hole_diameter: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PcbSmtPadElm {
    pub x: f64,
    pub y: f64,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub radius: Option<f64>,
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
    pub hole_diameter: Option<f64>,
    pub outer_diameter: Option<f64>,
    pub outer_width: Option<f64>,
    pub outer_height: Option<f64>,
    pub rect_pad_width: Option<f64>,
    pub rect_pad_height: Option<f64>,
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

impl PcbModel {
    pub fn parse(circuit_json: &str) -> Self {
        let mut model = Self::default();
        let elms: Vec<serde_json::Value> = serde_json::from_str(circuit_json).unwrap_or_default();

        for val in elms {
            let typ = match val.get("type").and_then(|t| t.as_str()) {
                Some(t) => t,
                None => continue,
            };
            match typ {
                "pcb_board" => {
                    if let Ok(b) = serde_json::from_value(val) {
                        model.boards.push(b);
                    }
                }
                "pcb_smtpad" => {
                    if let Ok(p) = serde_json::from_value(val) {
                        model.smt_pads.push(p);
                    }
                }
                "pcb_trace" => {
                    if let Ok(t) = serde_json::from_value(val) {
                        model.traces.push(t);
                    }
                }
                "pcb_plated_hole" => {
                    if let Ok(h) = serde_json::from_value(val) {
                        model.plated_holes.push(h);
                    }
                }
                "pcb_via" => {
                    if let Ok(v) = serde_json::from_value(val) {
                        model.vias.push(v);
                    }
                }
                "pcb_silkscreen_path" => {
                    if let Ok(s) = serde_json::from_value(val) {
                        model.silkscreen_paths.push(s);
                    }
                }
                "pcb_silkscreen_text" => {
                    if let Ok(st) = serde_json::from_value(val) {
                        model.silkscreen_texts.push(st);
                    }
                }
                "pcb_component" => {
                    if let Ok(c) = serde_json::from_value(val) {
                        model.components.push(c);
                    }
                }
                "pcb_hole" => {
                    if let Ok(h) = serde_json::from_value(val) {
                        model.holes.push(h);
                    }
                }
                _ => {}
            }
        }
        model
    }

    pub fn build_edge_cuts(&self) -> String {
        let mut dict = ApertureDictionary::new();
        let border_ap = dict.intern(Aperture::Circle {
            diameter_nm: 150_000, // 0.15mm outline stroke
        });

        let mut writer = GerberWriter::new("Edge Cuts (Board Outline)", "Profile,NP", true);
        writer.write_apertures(&dict);
        writer.select_aperture(border_ap);

        for board in &self.boards {
            let outline = board.outline_points();

            if let Some(first) = outline.first() {
                writer.move_to(Coord::from_mm(first.x), Coord::from_mm(first.y));
                for pt in &outline[1..] {
                    writer.draw_to(Coord::from_mm(pt.x), Coord::from_mm(pt.y));
                }
            }
        }

        writer.finish()
    }

    pub fn build_copper_layer(&self, is_top: bool) -> String {
        let layer_name = if is_top { "top" } else { "bottom" };
        let title = if is_top {
            "Top Copper (F_Cu)"
        } else {
            "Bottom Copper (B_Cu)"
        };
        let func = if is_top {
            "Copper,L1,Top"
        } else {
            "Copper,L2,Bot"
        };

        let mut dict = ApertureDictionary::new();

        // 1. Register apertures for smt pads
        let mut pad_apertures = Vec::new();
        for pad in self
            .smt_pads
            .iter()
            .filter(|p| p.layer.as_deref() == Some(layer_name))
        {
            let ap = make_pad_aperture(pad);
            let id = dict.intern(ap);
            pad_apertures.push((id, pad));
        }

        // 2. Register apertures for plated holes & vias
        let mut hole_apertures = Vec::new();
        for hole in &self.plated_holes {
            let d = hole.outer_diameter.unwrap_or(1.6);
            let id = dict.intern(Aperture::Circle {
                diameter_nm: (d * 1_000_000.0) as i64,
            });
            hole_apertures.push((id, hole.x, hole.y));
        }
        for via in &self.vias {
            let d = via.outer_diameter.unwrap_or(0.6);
            let id = dict.intern(Aperture::Circle {
                diameter_nm: (d * 1_000_000.0) as i64,
            });
            hole_apertures.push((id, via.x, via.y));
        }

        // 3. Register apertures for traces
        let mut trace_apertures = Vec::new();
        for trace in &self.traces {
            for win in trace.route.windows(2) {
                let p1 = &win[0];
                let p2 = &win[1];
                let pt_layer = p1.layer.as_deref().or(p2.layer.as_deref()).unwrap_or("top");
                if pt_layer == layer_name {
                    let w = p1.width.or(p2.width).unwrap_or(0.15);
                    let id = dict.intern(Aperture::Circle {
                        diameter_nm: (w * 1_000_000.0) as i64,
                    });
                    trace_apertures.push((id, p1.x, p1.y, p2.x, p2.y));
                }
            }
        }

        let mut writer = GerberWriter::new(title, func, true);
        writer.write_apertures(&dict);

        // Flash pads
        for (id, pad) in pad_apertures {
            writer.select_aperture(id);
            writer.flash(Coord::from_mm(pad.x), Coord::from_mm(pad.y));
        }

        // Flash hole copper rings
        for (id, x, y) in hole_apertures {
            writer.select_aperture(id);
            writer.flash(Coord::from_mm(x), Coord::from_mm(y));
        }

        // Draw traces
        for (id, x1, y1, x2, y2) in trace_apertures {
            writer.select_aperture(id);
            writer.move_to(Coord::from_mm(x1), Coord::from_mm(y1));
            writer.draw_to(Coord::from_mm(x2), Coord::from_mm(y2));
        }

        writer.finish()
    }

    pub fn build_mask_layer(&self, is_top: bool) -> String {
        let layer_name = if is_top { "top" } else { "bottom" };
        let title = if is_top {
            "Top Solder Mask (F_Mask)"
        } else {
            "Bottom Solder Mask (B_Mask)"
        };
        let func = if is_top {
            "Soldermask,Top"
        } else {
            "Soldermask,Bot"
        };

        let mut dict = ApertureDictionary::new();
        let mut mask_pads = Vec::new();

        for pad in self
            .smt_pads
            .iter()
            .filter(|p| p.layer.as_deref() == Some(layer_name))
        {
            if pad.is_covered_with_solder_mask == Some(true) {
                continue;
            }
            let margin = pad.soldermask_margin.unwrap_or(0.05);
            let ap = make_mask_pad_aperture(pad, margin);
            let id = dict.intern(ap);
            mask_pads.push((id, pad.x, pad.y));
        }

        for hole in &self.plated_holes {
            if hole.is_covered_with_solder_mask == Some(true) {
                continue;
            }
            let margin = hole.soldermask_margin.unwrap_or(0.05);
            let d = hole.outer_diameter.unwrap_or(1.6) + margin * 2.0;
            let id = dict.intern(Aperture::Circle {
                diameter_nm: (d * 1_000_000.0) as i64,
            });
            mask_pads.push((id, hole.x, hole.y));
        }

        // Vias are tented by default: only untented vias get mask openings
        for via in self.vias.iter().filter(|v| v.is_tented == Some(false)) {
            let d = via.outer_diameter.unwrap_or(0.6) + 0.1;
            let id = dict.intern(Aperture::Circle {
                diameter_nm: (d * 1_000_000.0) as i64,
            });
            mask_pads.push((id, via.x, via.y));
        }

        let mut writer = GerberWriter::new(title, func, true);
        writer.write_apertures(&dict);

        for (id, x, y) in mask_pads {
            writer.select_aperture(id);
            writer.flash(Coord::from_mm(x), Coord::from_mm(y));
        }

        writer.finish()
    }

    pub fn build_paste_layer(&self, is_top: bool) -> String {
        let layer_name = if is_top { "top" } else { "bottom" };
        let title = if is_top {
            "Top Solder Paste (F_Paste)"
        } else {
            "Bottom Solder Paste (B_Paste)"
        };
        let func = if is_top { "Paste,Top" } else { "Paste,Bot" };

        let mut dict = ApertureDictionary::new();
        let mut paste_pads = Vec::new();

        for pad in self
            .smt_pads
            .iter()
            .filter(|p| p.layer.as_deref() == Some(layer_name))
        {
            let ap = make_pad_aperture(pad);
            let id = dict.intern(ap);
            paste_pads.push((id, pad.x, pad.y));
        }

        let mut writer = GerberWriter::new(title, func, true);
        writer.write_apertures(&dict);

        for (id, x, y) in paste_pads {
            writer.select_aperture(id);
            writer.flash(Coord::from_mm(x), Coord::from_mm(y));
        }

        writer.finish()
    }

    pub fn build_silkscreen_layer(&self, is_top: bool) -> String {
        let layer_name = if is_top { "top" } else { "bottom" };
        let title = if is_top {
            "Top Silkscreen (F_Silk)"
        } else {
            "Bottom Silkscreen (B_Silk)"
        };
        let func = if is_top { "Legend,Top" } else { "Legend,Bot" };

        let mut dict = ApertureDictionary::new();
        let mut paths = Vec::new();

        for path in self
            .silkscreen_paths
            .iter()
            .filter(|p| p.layer.as_deref() == Some(layer_name) || (is_top && p.layer.is_none()))
        {
            let w = path.stroke_width.unwrap_or(0.15);
            let id = dict.intern(Aperture::Circle {
                diameter_nm: (w * 1_000_000.0) as i64,
            });
            paths.push((id, &path.route));
        }

        let mut writer = GerberWriter::new(title, func, true);
        writer.write_apertures(&dict);

        for (id, pts) in paths {
            if let Some(first) = pts.first() {
                writer.select_aperture(id);
                writer.move_to(Coord::from_mm(first.x), Coord::from_mm(first.y));
                for pt in &pts[1..] {
                    writer.draw_to(Coord::from_mm(pt.x), Coord::from_mm(pt.y));
                }
            }
        }

        writer.finish()
    }

    pub fn build_drill_file(&self, title: &str) -> String {
        let mut writer = ExcellonWriter::new(title, "Plated,1,2,PTH");

        for hole in &self.plated_holes {
            let d = hole.hole_diameter.unwrap_or(0.8);
            let tool = writer.register_tool(d);
            writer.add_hole(tool, Coord::from_mm(hole.x), Coord::from_mm(hole.y));
        }

        for via in &self.vias {
            let d = via.hole_diameter.unwrap_or(0.3);
            let tool = writer.register_tool(d);
            writer.add_hole(tool, Coord::from_mm(via.x), Coord::from_mm(via.y));
        }

        for trace in &self.traces {
            for pt in &trace.route {
                if pt.route_type.as_deref() == Some("via") {
                    let d = pt.hole_diameter.unwrap_or(0.3);
                    let tool = writer.register_tool(d);
                    writer.add_hole(tool, Coord::from_mm(pt.x), Coord::from_mm(pt.y));
                }
            }
        }

        writer.finish()
    }

    /// Builds the non-plated drill file for `pcb_hole` mounting holes, or `None` when the board has none.
    pub fn build_npth_drill_file(&self, title: &str) -> Option<String> {
        let mut writer = ExcellonWriter::new(title, "NonPlated,1,2,NPTH");
        for hole in &self.holes {
            let Some(d) = hole.hole_diameter.filter(|d| *d > 0.0) else {
                continue;
            };
            let tool = writer.register_tool(d);
            writer.add_hole(tool, Coord::from_mm(hole.x), Coord::from_mm(hole.y));
        }
        (!writer.is_empty()).then(|| writer.finish())
    }
}

fn make_pad_aperture(pad: &PcbSmtPadElm) -> Aperture {
    let shape = pad.shape.as_deref().unwrap_or("rect");
    match shape {
        "circle" => {
            let d = pad.radius.map(|r| r * 2.0).or(pad.width).unwrap_or(1.0);
            Aperture::Circle {
                diameter_nm: (d * 1_000_000.0) as i64,
            }
        }
        "pill" | "rotated_pill" | "oval" => {
            let w = pad.width.unwrap_or(1.0);
            let h = pad.height.unwrap_or(0.5);
            Aperture::Obround {
                width_nm: (w * 1_000_000.0) as i64,
                height_nm: (h * 1_000_000.0) as i64,
            }
        }
        _ => {
            let w = pad.width.unwrap_or(1.0);
            let h = pad.height.unwrap_or(0.5);
            Aperture::Rectangle {
                width_nm: (w * 1_000_000.0) as i64,
                height_nm: (h * 1_000_000.0) as i64,
            }
        }
    }
}

fn make_mask_pad_aperture(pad: &PcbSmtPadElm, margin: f64) -> Aperture {
    let shape = pad.shape.as_deref().unwrap_or("rect");
    match shape {
        "circle" => {
            let d = pad.radius.map(|r| r * 2.0).or(pad.width).unwrap_or(1.0) + margin * 2.0;
            Aperture::Circle {
                diameter_nm: (d * 1_000_000.0) as i64,
            }
        }
        "pill" | "rotated_pill" | "oval" => {
            let w = pad.width.unwrap_or(1.0) + margin * 2.0;
            let h = pad.height.unwrap_or(0.5) + margin * 2.0;
            Aperture::Obround {
                width_nm: (w * 1_000_000.0) as i64,
                height_nm: (h * 1_000_000.0) as i64,
            }
        }
        _ => {
            let w = pad.width.unwrap_or(1.0) + margin * 2.0;
            let h = pad.height.unwrap_or(0.5) + margin * 2.0;
            Aperture::Rectangle {
                width_nm: (w * 1_000_000.0) as i64,
                height_nm: (h * 1_000_000.0) as i64,
            }
        }
    }
}
