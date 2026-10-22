//! Layer builders mapping Circuit JSON elements to RS-274X and Excellon files.

use std::collections::BTreeSet;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

use crate::gerber::apertures::{Aperture, ApertureDictionary};
use crate::gerber::coordinates::Coord;
use crate::gerber::excellon::ExcellonWriter;
use crate::gerber::writer::{FilePolarity, GerberWriter};

const EDGE_STROKE_NM: i64 = 150_000;
const DEFAULT_MASK_MARGIN_MM: f64 = 0.05;
const DEFAULT_SILK_STROKE_MM: f64 = 0.15;

/// Reasons a Circuit JSON document cannot be turned into fabrication files.
#[derive(Debug, Error)]
pub enum GerberError {
    #[error("invalid Circuit JSON: {0}")]
    InvalidJson(String),
    #[error("Circuit JSON contains no pcb_board; refusing to emit fabrication files")]
    NoBoard,
    #[error("malformed {kind} at JSON index {index}: {reason}")]
    Malformed {
        kind: String,
        index: usize,
        reason: String,
    },
    #[error("invalid {kind} #{index}: {reason}")]
    Invalid {
        kind: &'static str,
        index: usize,
        reason: String,
    },
}

impl From<GerberError> for std::io::Error {
    fn from(err: GerberError) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
    }
}

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
    pub hole_shape: Option<String>,
    pub hole_diameter: Option<f64>,
    pub hole_width: Option<f64>,
    pub hole_height: Option<f64>,
    pub ccw_rotation: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PcbSmtPadElm {
    pub x: f64,
    pub y: f64,
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

#[derive(Debug, Clone, Copy)]
enum PadShape {
    Circle { d: f64 },
    Rect { w: f64, h: f64, rot: f64 },
    Pill { w: f64, h: f64, rot: f64 },
}

#[derive(Debug, Clone, Copy)]
enum Drill {
    Round {
        d: f64,
        at: [f64; 2],
    },
    Slot {
        w: f64,
        h: f64,
        rot: f64,
        at: [f64; 2],
    },
}

#[derive(Debug)]
enum Feature {
    Flash(Aperture, [f64; 2]),
    Stroke(Aperture, [f64; 2], [f64; 2]),
    Region(Vec<[f64; 2]>),
}

#[derive(Debug, Clone, Copy)]
struct ViaSite {
    at: [f64; 2],
    outer: f64,
    hole: f64,
    tented: bool,
}

#[derive(Debug)]
struct TraceSegment {
    is_top: bool,
    width: f64,
    from: [f64; 2],
    to: [f64; 2],
}

impl PcbModel {
    /// Leniently parses Circuit JSON, skipping elements that fail to deserialize. For previews only, never for fabrication.
    pub fn parse(circuit_json: &str) -> Self {
        let mut model = Self::default();
        let elms: Vec<Value> = serde_json::from_str(circuit_json).unwrap_or_default();
        for (index, val) in elms.into_iter().enumerate() {
            let _ = model.push(index, val);
        }
        model
    }

    /// Strictly parses Circuit JSON for fabrication: invalid JSON, malformed PCB elements, or a missing board are errors.
    pub fn try_parse(circuit_json: &str) -> Result<Self, GerberError> {
        let elms: Vec<Value> = serde_json::from_str(circuit_json)
            .map_err(|e| GerberError::InvalidJson(e.to_string()))?;
        let mut model = Self::default();
        for (index, val) in elms.into_iter().enumerate() {
            model.push(index, val)?;
        }
        if model.boards.is_empty() {
            return Err(GerberError::NoBoard);
        }
        Ok(model)
    }

    fn push(&mut self, index: usize, val: Value) -> Result<(), GerberError> {
        let Some(typ) = val.get("type").and_then(Value::as_str).map(str::to_owned) else {
            return Ok(());
        };
        match typ.as_str() {
            "pcb_board" => self.boards.push(decode(&typ, index, val)?),
            "pcb_smtpad" => self.smt_pads.push(decode(&typ, index, val)?),
            "pcb_trace" => self.traces.push(decode(&typ, index, val)?),
            "pcb_plated_hole" => self.plated_holes.push(decode(&typ, index, val)?),
            "pcb_via" => self.vias.push(decode(&typ, index, val)?),
            "pcb_silkscreen_path" => self.silkscreen_paths.push(decode(&typ, index, val)?),
            "pcb_silkscreen_text" => self.silkscreen_texts.push(decode(&typ, index, val)?),
            "pcb_component" => self.components.push(decode(&typ, index, val)?),
            "pcb_hole" => self.holes.push(decode(&typ, index, val)?),
            _ => {}
        }
        Ok(())
    }

    /// Builds the board profile, closing every outline back to its first vertex.
    pub fn build_edge_cuts(&self) -> Result<String, GerberError> {
        let mut rings = Vec::new();
        for (i, board) in self.boards.iter().enumerate() {
            rings.push(closed_outline(board).map_err(invalid("pcb_board", i))?);
        }

        let mut dict = ApertureDictionary::new();
        let border_ap = dict.intern(Aperture::Circle {
            diameter_nm: EDGE_STROKE_NM,
        });
        let mut writer = GerberWriter::new(
            "Edge Cuts (Board Outline)",
            "Profile,NP",
            FilePolarity::Positive,
            &dict,
        );
        writer.select_aperture(border_ap);
        for ring in rings {
            let [fx, fy] = ring[0];
            writer.move_to(fx, fy);
            for &[x, y] in &ring[1..] {
                writer.draw_to(x, y);
            }
        }
        Ok(writer.finish())
    }

    /// Builds a copper layer: SMT pads, plated hole rings, via pads and trace strokes.
    pub fn build_copper_layer(&self, is_top: bool) -> Result<String, GerberError> {
        let mut features = Vec::new();
        for (i, pad) in self.smt_pads.iter().enumerate() {
            let feature = (|| {
                if pad_is_top(pad)? != is_top {
                    return Ok(None);
                }
                smtpad_shape(pad)?.feature([pad.x, pad.y], 0.0).map(Some)
            })()
            .map_err(invalid("pcb_smtpad", i))?;
            features.extend(feature);
        }
        for (i, hole) in self.plated_holes.iter().enumerate() {
            features.push(
                plated_hole_geometry(hole)
                    .and_then(|(shape, _)| shape.feature([hole.x, hole.y], 0.0))
                    .map_err(invalid("pcb_plated_hole", i))?,
            );
        }
        for (i, via) in self.via_sites()?.into_iter().enumerate() {
            features.push(
                PadShape::Circle { d: via.outer }
                    .feature(via.at, 0.0)
                    .map_err(invalid("via", i))?,
            );
        }
        for (i, seg) in self.trace_segments()?.into_iter().enumerate() {
            if seg.is_top == is_top {
                let ap = Aperture::circle(seg.width).map_err(invalid("pcb_trace segment", i))?;
                features.push(Feature::Stroke(ap, seg.from, seg.to));
            }
        }

        let (title, func) = if is_top {
            ("Top Copper (F_Cu)", "Copper,L1,Top")
        } else {
            ("Bottom Copper (B_Cu)", "Copper,L2,Bot")
        };
        Ok(render(title, func, FilePolarity::Positive, &features))
    }

    /// Builds a negative-polarity solder mask layer whose image is the set of mask openings.
    pub fn build_mask_layer(&self, is_top: bool) -> Result<String, GerberError> {
        let mut features = Vec::new();
        for (i, pad) in self.smt_pads.iter().enumerate() {
            let feature = (|| {
                if pad_is_top(pad)? != is_top || pad.is_covered_with_solder_mask == Some(true) {
                    return Ok(None);
                }
                let margin = finite_or(
                    pad.soldermask_margin,
                    DEFAULT_MASK_MARGIN_MM,
                    "soldermask_margin",
                )?;
                smtpad_shape(pad)?.feature([pad.x, pad.y], margin).map(Some)
            })()
            .map_err(invalid("pcb_smtpad", i))?;
            features.extend(feature);
        }
        for (i, hole) in self.plated_holes.iter().enumerate() {
            if hole.is_covered_with_solder_mask == Some(true) {
                continue;
            }
            features.push(
                (|| {
                    let margin = finite_or(
                        hole.soldermask_margin,
                        DEFAULT_MASK_MARGIN_MM,
                        "soldermask_margin",
                    )?;
                    plated_hole_geometry(hole)?
                        .0
                        .feature([hole.x, hole.y], margin)
                })()
                .map_err(invalid("pcb_plated_hole", i))?,
            );
        }
        for (i, via) in self.via_sites()?.into_iter().enumerate() {
            if !via.tented {
                features.push(
                    PadShape::Circle { d: via.outer }
                        .feature(via.at, DEFAULT_MASK_MARGIN_MM)
                        .map_err(invalid("via", i))?,
                );
            }
        }

        let (title, func) = if is_top {
            ("Top Solder Mask (F_Mask)", "Soldermask,Top")
        } else {
            ("Bottom Solder Mask (B_Mask)", "Soldermask,Bot")
        };
        Ok(render(title, func, FilePolarity::Negative, &features))
    }

    /// Builds a solder paste layer from the exposed SMT pads on one side.
    pub fn build_paste_layer(&self, is_top: bool) -> Result<String, GerberError> {
        let mut features = Vec::new();
        for (i, pad) in self.smt_pads.iter().enumerate() {
            let feature = (|| {
                if pad_is_top(pad)? != is_top || pad.is_covered_with_solder_mask == Some(true) {
                    return Ok(None);
                }
                smtpad_shape(pad)?.feature([pad.x, pad.y], 0.0).map(Some)
            })()
            .map_err(invalid("pcb_smtpad", i))?;
            features.extend(feature);
        }

        let (title, func) = if is_top {
            ("Top Solder Paste (F_Paste)", "Paste,Top")
        } else {
            ("Bottom Solder Paste (B_Paste)", "Paste,Bot")
        };
        Ok(render(title, func, FilePolarity::Positive, &features))
    }

    /// Builds a silkscreen layer from silkscreen paths.
    pub fn build_silkscreen_layer(&self, is_top: bool) -> Result<String, GerberError> {
        let layer_name = if is_top { "top" } else { "bottom" };
        let mut features = Vec::new();
        for (i, path) in self.silkscreen_paths.iter().enumerate() {
            let on_side =
                path.layer.as_deref() == Some(layer_name) || (is_top && path.layer.is_none());
            let width = path.stroke_width.unwrap_or(DEFAULT_SILK_STROKE_MM);
            if !on_side || width == 0.0 {
                continue;
            }
            let ap = Aperture::circle(width).map_err(invalid("pcb_silkscreen_path", i))?;
            for win in path.route.windows(2) {
                features.push(Feature::Stroke(
                    ap.clone(),
                    [win[0].x, win[0].y],
                    [win[1].x, win[1].y],
                ));
            }
        }

        let (title, func) = if is_top {
            ("Top Silkscreen (F_Silk)", "Legend,Top")
        } else {
            ("Bottom Silkscreen (B_Silk)", "Legend,Bot")
        };
        Ok(render(title, func, FilePolarity::Positive, &features))
    }

    /// Builds the plated drill file: plated holes and slots plus every distinct via.
    pub fn build_drill_file(&self, title: &str) -> Result<String, GerberError> {
        let mut writer = ExcellonWriter::new(title, "Plated,1,2,PTH");
        for (i, hole) in self.plated_holes.iter().enumerate() {
            plated_hole_geometry(hole)
                .and_then(|(_, drill)| drill.emit(&mut writer))
                .map_err(invalid("pcb_plated_hole", i))?;
        }
        for (i, via) in self.via_sites()?.into_iter().enumerate() {
            Drill::Round {
                d: via.hole,
                at: via.at,
            }
            .emit(&mut writer)
            .map_err(invalid("via", i))?;
        }
        Ok(writer.finish())
    }

    /// Builds the non-plated drill file from `pcb_hole` elements, or `None` when the board has none.
    pub fn build_npth_drill_file(&self, title: &str) -> Result<Option<String>, GerberError> {
        let mut writer = ExcellonWriter::new(title, "NonPlated,1,2,NPTH");
        for (i, hole) in self.holes.iter().enumerate() {
            hole_drill(hole)
                .and_then(|drill| drill.emit(&mut writer))
                .map_err(invalid("pcb_hole", i))?;
        }
        Ok((!writer.is_empty()).then(|| writer.finish()))
    }

    fn via_sites(&self) -> Result<Vec<ViaSite>, GerberError> {
        let mut seen = BTreeSet::new();
        let mut sites = Vec::new();
        for (i, via) in self.vias.iter().enumerate() {
            let site = (|| {
                Ok::<_, String>(ViaSite {
                    at: [via.x, via.y],
                    outer: required(via.outer_diameter, "outer_diameter")?,
                    hole: required(via.hole_diameter, "hole_diameter")?,
                    tented: via.is_tented != Some(false),
                })
            })()
            .map_err(invalid("pcb_via", i))?;
            seen.insert(coord(site.at));
            sites.push(site);
        }
        for (i, trace) in self.traces.iter().enumerate() {
            for pt in &trace.route {
                if pt.route_type.as_deref() != Some("via") || !seen.insert(coord([pt.x, pt.y])) {
                    continue;
                }
                let site = (|| {
                    Ok::<_, String>(ViaSite {
                        at: [pt.x, pt.y],
                        outer: required(
                            pt.outer_diameter,
                            "via outer_diameter (no pcb_via at this position)",
                        )?,
                        hole: required(
                            pt.hole_diameter,
                            "via hole_diameter (no pcb_via at this position)",
                        )?,
                        tented: true,
                    })
                })()
                .map_err(invalid("pcb_trace", i))?;
                sites.push(site);
            }
        }
        Ok(sites)
    }

    fn trace_segments(&self) -> Result<Vec<TraceSegment>, GerberError> {
        let mut segments = Vec::new();
        for (i, trace) in self.traces.iter().enumerate() {
            for win in trace.route.windows(2) {
                segments.push(trace_segment(&win[0], &win[1]).map_err(invalid("pcb_trace", i))?);
            }
        }
        Ok(segments)
    }
}

impl PadShape {
    fn feature(self, at: [f64; 2], margin: f64) -> Result<Feature, String> {
        match self {
            Self::Circle { d } => Ok(Feature::Flash(Aperture::circle(d + 2.0 * margin)?, at)),
            Self::Rect { w, h, rot } => {
                let (w, h) = (w + 2.0 * margin, h + 2.0 * margin);
                match quarter_turns(rot) {
                    Some(q) => {
                        let (w, h) = if q % 2 == 1 { (h, w) } else { (w, h) };
                        Ok(Feature::Flash(Aperture::rectangle(w, h)?, at))
                    }
                    None => {
                        Aperture::rectangle(w, h)?;
                        let corners = [[-w, -h], [w, -h], [w, h], [-w, h]]
                            .map(|[cx, cy]| add(at, rotate([cx / 2.0, cy / 2.0], rot)));
                        Ok(Feature::Region(corners.to_vec()))
                    }
                }
            }
            Self::Pill { w, h, rot } => {
                let (w, h) = (w + 2.0 * margin, h + 2.0 * margin);
                match quarter_turns(rot) {
                    Some(q) => {
                        let (w, h) = if q % 2 == 1 { (h, w) } else { (w, h) };
                        Ok(Feature::Flash(Aperture::obround(w, h)?, at))
                    }
                    None => {
                        Aperture::obround(w, h)?;
                        let (from, to) = long_axis(w, h, rot, at);
                        Ok(Feature::Stroke(Aperture::circle(w.min(h))?, from, to))
                    }
                }
            }
        }
    }
}

impl Drill {
    fn emit(self, writer: &mut ExcellonWriter) -> Result<(), String> {
        match self {
            Self::Round { d, at } => {
                let tool = writer.register_tool(d)?;
                writer.add_hole(tool, Coord::from_mm(at[0]), Coord::from_mm(at[1]));
            }
            Self::Slot { w, h, rot, at } => {
                let tool = writer.register_tool(w.min(h))?;
                let (from, to) = long_axis(w, h, rot, at);
                if coord(from) == coord(to) {
                    writer.add_hole(tool, Coord::from_mm(at[0]), Coord::from_mm(at[1]));
                } else {
                    writer.add_slot(tool, coord(from), coord(to));
                }
            }
        }
        Ok(())
    }
}

fn decode<T: DeserializeOwned>(kind: &str, index: usize, val: Value) -> Result<T, GerberError> {
    serde_json::from_value(val).map_err(|e| GerberError::Malformed {
        kind: kind.to_string(),
        index,
        reason: e.to_string(),
    })
}

fn invalid(kind: &'static str, index: usize) -> impl Fn(String) -> GerberError {
    move |reason| GerberError::Invalid {
        kind,
        index,
        reason,
    }
}

fn render(title: &str, func: &str, polarity: FilePolarity, features: &[Feature]) -> String {
    let mut dict = ApertureDictionary::new();
    let ids: Vec<Option<u32>> = features
        .iter()
        .map(|f| match f {
            Feature::Flash(ap, _) | Feature::Stroke(ap, _, _) => Some(dict.intern(ap.clone())),
            Feature::Region(_) => None,
        })
        .collect();
    let mut writer = GerberWriter::new(title, func, polarity, &dict);
    for (feature, id) in features.iter().zip(ids) {
        if let Some(id) = id {
            writer.select_aperture(id);
        }
        match feature {
            Feature::Flash(_, [x, y]) => writer.flash(Coord::from_mm(*x), Coord::from_mm(*y)),
            Feature::Stroke(_, [x1, y1], [x2, y2]) => {
                writer.move_to(Coord::from_mm(*x1), Coord::from_mm(*y1));
                writer.draw_to(Coord::from_mm(*x2), Coord::from_mm(*y2));
            }
            Feature::Region(points) => {
                let pts: Vec<[Coord; 2]> = points.iter().map(|p| coord(*p)).collect();
                writer.region(&pts);
            }
        }
    }
    writer.finish()
}

fn closed_outline(board: &PcbBoardElm) -> Result<Vec<[Coord; 2]>, String> {
    match &board.outline {
        Some(points) if points.len() < 3 => {
            return Err(format!(
                "outline has {} point(s), needs at least 3",
                points.len()
            ))
        }
        Some(_) => {}
        None => {
            if required(board.width, "width")? <= 0.0 || required(board.height, "height")? <= 0.0 {
                return Err("width and height must be positive".to_string());
            }
        }
    }
    let points = board.outline_points();
    if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
        return Err("outline contains a non-finite coordinate".to_string());
    }
    let mut ring: Vec<[Coord; 2]> = points.iter().map(|p| coord([p.x, p.y])).collect();
    if ring.first() != ring.last() {
        ring.push(ring[0]);
    }
    Ok(ring)
}

fn pad_is_top(pad: &PcbSmtPadElm) -> Result<bool, String> {
    layer_is_top(pad.layer.as_deref())
}

fn layer_is_top(layer: Option<&str>) -> Result<bool, String> {
    match layer {
        Some("top") => Ok(true),
        Some("bottom") => Ok(false),
        Some(other) => Err(format!("unsupported copper layer '{other}'")),
        None => Err("missing layer".to_string()),
    }
}

fn smtpad_shape(pad: &PcbSmtPadElm) -> Result<PadShape, String> {
    let rot = finite_or(pad.ccw_rotation, 0.0, "ccw_rotation")?;
    let w = || required(pad.width, "width");
    let h = || required(pad.height, "height");
    match pad.shape.as_deref() {
        Some("rect") => Ok(PadShape::Rect {
            w: w()?,
            h: h()?,
            rot,
        }),
        Some("rotated_rect") => Ok(PadShape::Rect {
            w: w()?,
            h: h()?,
            rot: required(pad.ccw_rotation, "ccw_rotation")?,
        }),
        Some("circle") => Ok(PadShape::Circle {
            d: required(pad.radius, "radius")? * 2.0,
        }),
        Some("pill") | Some("oval") => Ok(PadShape::Pill {
            w: w()?,
            h: h()?,
            rot,
        }),
        Some("rotated_pill") => Ok(PadShape::Pill {
            w: w()?,
            h: h()?,
            rot: required(pad.ccw_rotation, "ccw_rotation")?,
        }),
        Some(other) => Err(format!("unsupported pad shape '{other}'")),
        None => Err("missing shape".to_string()),
    }
}

fn plated_hole_geometry(hole: &PcbPlatedHoleElm) -> Result<(PadShape, Drill), String> {
    let center = [hole.x, hole.y];
    let offset = [
        finite_or(hole.hole_offset_x, 0.0, "hole_offset_x")?,
        finite_or(hole.hole_offset_y, 0.0, "hole_offset_y")?,
    ];
    let rect_pad = |rot: f64| -> Result<PadShape, String> {
        Ok(PadShape::Rect {
            w: required(hole.rect_pad_width, "rect_pad_width")?,
            h: required(hole.rect_pad_height, "rect_pad_height")?,
            rot,
        })
    };
    let slot = |rot: f64, at: [f64; 2]| -> Result<Drill, String> {
        Ok(Drill::Slot {
            w: required(hole.hole_width, "hole_width")?,
            h: required(hole.hole_height, "hole_height")?,
            rot,
            at,
        })
    };
    match hole.shape.as_deref() {
        Some("circle") => Ok((
            PadShape::Circle {
                d: required(hole.outer_diameter, "outer_diameter")?,
            },
            Drill::Round {
                d: required(hole.hole_diameter, "hole_diameter")?,
                at: center,
            },
        )),
        Some("oval") | Some("pill") => {
            let rot = finite_or(hole.ccw_rotation, 0.0, "ccw_rotation")?;
            Ok((
                PadShape::Pill {
                    w: required(hole.outer_width, "outer_width")?,
                    h: required(hole.outer_height, "outer_height")?,
                    rot,
                },
                slot(rot, center)?,
            ))
        }
        Some("circular_hole_with_rect_pad") => Ok((
            rect_pad(finite_or(hole.rect_ccw_rotation, 0.0, "rect_ccw_rotation")?)?,
            Drill::Round {
                d: required(hole.hole_diameter, "hole_diameter")?,
                at: add(center, offset),
            },
        )),
        Some("pill_hole_with_rect_pad") => Ok((rect_pad(0.0)?, slot(0.0, add(center, offset))?)),
        Some("rotated_pill_hole_with_rect_pad") => Ok((
            rect_pad(required(hole.rect_ccw_rotation, "rect_ccw_rotation")?)?,
            slot(
                required(hole.hole_ccw_rotation, "hole_ccw_rotation")?,
                add(center, offset),
            )?,
        )),
        Some(other) => Err(format!("unsupported plated hole shape '{other}'")),
        None => Err("missing shape".to_string()),
    }
}

fn hole_drill(hole: &PcbHoleElm) -> Result<Drill, String> {
    let at = [hole.x, hole.y];
    let slot = |rot: f64| -> Result<Drill, String> {
        Ok(Drill::Slot {
            w: required(hole.hole_width, "hole_width")?,
            h: required(hole.hole_height, "hole_height")?,
            rot,
            at,
        })
    };
    match hole.hole_shape.as_deref() {
        Some("circle") => Ok(Drill::Round {
            d: required(hole.hole_diameter, "hole_diameter")?,
            at,
        }),
        Some("oval") | Some("pill") => slot(finite_or(hole.ccw_rotation, 0.0, "ccw_rotation")?),
        Some("rotated_pill") => slot(required(hole.ccw_rotation, "ccw_rotation")?),
        Some(other) => Err(format!("unsupported hole shape '{other}'")),
        None => Err("missing hole_shape".to_string()),
    }
}

fn trace_segment(p1: &PcbTracePoint, p2: &PcbTracePoint) -> Result<TraceSegment, String> {
    let wire = |p: &PcbTracePoint| -> Result<Option<(bool, f64)>, String> {
        match p.route_type.as_deref() {
            Some("wire") => Ok(Some((
                layer_is_top(p.layer.as_deref())?,
                required(p.width, "wire width")?,
            ))),
            Some("via") => Ok(None),
            Some(other) => Err(format!("unsupported route_type '{other}'")),
            None => Err("route point missing route_type".to_string()),
        }
    };
    let (is_top, width) = match (wire(p1)?, wire(p2)?) {
        (Some((l1, w)), Some((l2, _))) if l1 == l2 => (l1, w),
        (Some(_), Some(_)) => return Err("wire changes layer without a via".to_string()),
        (Some(a), None) | (None, Some(a)) => a,
        (None, None) => return Err("segment between two vias has no layer".to_string()),
    };
    Ok(TraceSegment {
        is_top,
        width,
        from: [p1.x, p1.y],
        to: [p2.x, p2.y],
    })
}

fn required(value: Option<f64>, name: &str) -> Result<f64, String> {
    match value {
        Some(v) if v.is_finite() => Ok(v),
        Some(v) => Err(format!("{name} is not finite ({v})")),
        None => Err(format!("missing {name}")),
    }
}

fn finite_or(value: Option<f64>, default: f64, name: &str) -> Result<f64, String> {
    value.map_or(Ok(default), |v| required(Some(v), name))
}

fn quarter_turns(rot: f64) -> Option<u8> {
    let q = rot / 90.0;
    let r = q.round();
    ((q - r).abs() < 1e-9).then(|| r.rem_euclid(4.0) as u8)
}

fn rotate([x, y]: [f64; 2], deg: f64) -> [f64; 2] {
    let (s, c) = deg.to_radians().sin_cos();
    [x * c - y * s, x * s + y * c]
}

fn add(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

fn long_axis(w: f64, h: f64, rot: f64, at: [f64; 2]) -> ([f64; 2], [f64; 2]) {
    let half = (w.max(h) - w.min(h)) / 2.0;
    let axis = if w >= h { [half, 0.0] } else { [0.0, half] };
    let [dx, dy] = rotate(axis, rot);
    ([at[0] - dx, at[1] - dy], [at[0] + dx, at[1] + dy])
}

fn coord([x, y]: [f64; 2]) -> [Coord; 2] {
    [Coord::from_mm(x), Coord::from_mm(y)]
}
