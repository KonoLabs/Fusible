//! `trace_clearance`: physical copper clearance design rule check (DRC).

use crate::linter::{Diagnostic, LintContext, Rule};
use crate::parser::{Pad, PadPoint};

/// Minimum copper clearance in millimeters (standard 6 mil / 0.15mm PCB manufacturing limit).
const MIN_CLEARANCE_MM: f64 = 0.15;
const CLEARANCE_HELP: &str = "reroute copper or relocate pads to maintain at least 0.15mm \
                              clearance to prevent manufacturing defects";

/// Inspects physical PCB copper geometry (pads, traces, vias) and detects
/// short circuits and clearance violations between distinct electrical nets.
pub struct TraceClearanceRule;

#[derive(Debug)]
struct CopperFeature {
    pub net_id: String,
    pub net_name: String,
    pub layer: String,
    pub x: f64,
    pub y: f64,
    pub half_w: f64,
    pub half_h: f64,
}

impl Rule for TraceClearanceRule {
    fn name(&self) -> &'static str {
        "trace_clearance"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let features = extract_copper_features(ctx);
        let mut diagnostics = Vec::new();
        for (i, f1) in features.iter().enumerate() {
            for f2 in &features[i + 1..] {
                diagnostics.extend(clearance_violation(f1, f2).map(|message| Diagnostic {
                    rule: self.name().to_string(),
                    message,
                    help: Some(CLEARANCE_HELP.to_string()),
                }));
            }
        }
        diagnostics
    }
}

/// The violation message when two copper features of distinct nets on one
/// layer sit closer than [`MIN_CLEARANCE_MM`].
fn clearance_violation(f1: &CopperFeature, f2: &CopperFeature) -> Option<String> {
    if f1.net_id == f2.net_id || !f1.layer.eq_ignore_ascii_case(&f2.layer) {
        return None;
    }
    let dx = (f1.x - f2.x).abs();
    let dy = (f1.y - f2.y).abs();
    let reach_x = f1.half_w + f2.half_w;
    let reach_y = f1.half_h + f2.half_h;
    let clearance_gap = ((dx - reach_x).max(0.0)).hypot((dy - reach_y).max(0.0));
    (clearance_gap < MIN_CLEARANCE_MM).then(|| {
        if dx < reach_x && dy < reach_y {
            format!(
                "direct copper short-circuit on {}: net \"{}\" and net \"{}\" overlap at \
                 ({:.2}, {:.2})",
                f1.layer, f1.net_name, f2.net_name, f1.x, f1.y
            )
        } else {
            format!(
                "copper clearance violation on {}: net \"{}\" and net \"{}\" have only \
                 {:.3}mm spacing (minimum {MIN_CLEARANCE_MM}mm)",
                f1.layer, f1.net_name, f2.net_name, clearance_gap
            )
        }
    })
}

/// Half extents of a pad: `width`/`height`, or `radius` for circle pads.
fn half_extents(pad: &Pad) -> (f64, f64) {
    match (pad.width, pad.height, pad.radius) {
        (None, None, Some(r)) => (r, r),
        (w, h, _) => (w.unwrap_or(1.0) / 2.0, h.unwrap_or(1.0) / 2.0),
    }
}

/// Center and half extents of a pad: from `x`/`y` and its size, or from the
/// bounding box of its polygon outline, which covers all of its copper.
fn pad_box(pad: &Pad) -> Option<(f64, f64, f64, f64)> {
    if let (Some(x), Some(y)) = (pad.x, pad.y) {
        let (half_w, half_h) = half_extents(pad);
        return Some((x, y, half_w, half_h));
    }
    let points = pad.points.as_deref().filter(|p| !p.is_empty())?;
    let fold = |f: fn(f64, f64) -> f64, init: f64, axis: fn(&PadPoint) -> f64| {
        points.iter().map(axis).fold(init, f)
    };
    let (min_x, max_x) = (
        fold(f64::min, f64::MAX, |p| p.x),
        fold(f64::max, f64::MIN, |p| p.x),
    );
    let (min_y, max_y) = (
        fold(f64::min, f64::MAX, |p| p.y),
        fold(f64::max, f64::MIN, |p| p.y),
    );
    Some((
        (min_x + max_x) / 2.0,
        (min_y + max_y) / 2.0,
        (max_x - min_x) / 2.0,
        (max_y - min_y) / 2.0,
    ))
}

/// Every positioned pad as copper. A pad whose net the netlist cannot
/// resolve is isolated copper: it gets a net of its own, so any overlap with
/// another pad is still a violation.
fn extract_copper_features(ctx: &LintContext<'_>) -> Vec<CopperFeature> {
    ctx.circuit
        .pads()
        .enumerate()
        .filter_map(|(index, pad)| {
            let (x, y, half_w, half_h) = pad_box(pad)?;
            let label = pad.pad_id.clone().unwrap_or_else(|| format!("#{index}"));
            let (net_id, net_name) = match pad
                .pad_id
                .as_deref()
                .and_then(|id| ctx.netlist.pad_to_net.get(id))
                .and_then(|&idx| ctx.netlist.nets.get(idx))
            {
                Some(node) => (node.id.clone(), node.display_name().to_string()),
                None => (
                    format!("unresolved pad {label}"),
                    format!("<unresolved pad {label}>"),
                ),
            };
            Some(CopperFeature {
                net_id,
                net_name,
                layer: pad.layer.clone().unwrap_or_else(|| "top".to_string()),
                x,
                y,
                half_w,
                half_h,
            })
        })
        .collect()
}
