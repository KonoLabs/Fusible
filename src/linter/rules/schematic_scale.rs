//! `schematic_scale`: checks for out-of-bounds schematic coordinates.

mod element;

use std::collections::HashSet;

use crate::linter::{Diagnostic, LintContext, Rule, Severity};
use element::{detect_multiple_sheets, extract_coord, resolve_refdes};

const MAX_CENTER_COORD_MM: f64 = 150.0;
const MAX_SPAN_MM: f64 = 300.0;
const SCALE_HELP: &str = "tscircuit schematic coordinates (schX, schY) are in millimeters, not \
                          screen pixels. A typical chip is 2-4mm across. Values in the hundreds \
                          cause the schematic canvas to zoom out into microscopic scale. Rescale \
                          coordinates to single-digit millimeter values.";

/// Verifies that schematic components are placed within standard schematic sheet
/// boundaries and alerts on millimeter/pixel coordinate scaling confusion.
pub struct SchematicScaleRule;

/// A schematic component's designator and center.
struct Placement {
    refdes: String,
    x: f64,
    y: f64,
}

/// The bounding box of every placement and whether its span overflows one sheet.
struct Bounds {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    span_exceeded: bool,
}

impl Rule for SchematicScaleRule {
    fn name(&self) -> &'static str {
        "E_SCHEMATIC_SCALE_EXCESSIVE"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let placements = collect_placements(ctx);
        if placements.is_empty() {
            return Vec::new();
        }
        let bounds = Bounds::of(&placements, detect_multiple_sheets(ctx));
        let mut reported = HashSet::new();
        placements
            .iter()
            .filter(|p| bounds.is_outside(p) && reported.insert(p.refdes.clone()))
            .map(|Placement { refdes, x, y }| Diagnostic {
                rule: self.name().to_string(),
                message: format!(
                    "Component '{refdes}' placed at ({x:.1}, {y:.1}) exceeds standard schematic \
                     sheet bounds"
                ),
                help: Some(SCALE_HELP.to_string()),
                ..Default::default()
            })
            .collect()
    }
}

impl Bounds {
    fn of(placements: &[Placement], has_multiple_sheets: bool) -> Self {
        let mut b = Self {
            min_x: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            min_y: f64::INFINITY,
            max_y: f64::NEG_INFINITY,
            span_exceeded: false,
        };
        for p in placements {
            b.min_x = b.min_x.min(p.x);
            b.max_x = b.max_x.max(p.x);
            b.min_y = b.min_y.min(p.y);
            b.max_y = b.max_y.max(p.y);
        }
        b.span_exceeded =
            !has_multiple_sheets && (b.span_x() > MAX_SPAN_MM || b.span_y() > MAX_SPAN_MM);
        b
    }

    fn span_x(&self) -> f64 {
        self.max_x - self.min_x
    }

    fn span_y(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// True when the placement leaves the sheet center window or sits on an
    /// edge of an overflowing span.
    fn is_outside(&self, p: &Placement) -> bool {
        let exceeds_center = !p.x.is_finite()
            || !p.y.is_finite()
            || p.x.abs() > MAX_CENTER_COORD_MM
            || p.y.abs() > MAX_CENTER_COORD_MM;
        let exceeds_span = self.span_exceeded
            && ((self.span_x() > MAX_SPAN_MM && (p.x == self.min_x || p.x == self.max_x))
                || (self.span_y() > MAX_SPAN_MM && (p.y == self.min_y || p.y == self.max_y)));
        exceeds_center || exceeds_span
    }
}

/// Every `schematic_component` with both coordinates.
fn collect_placements(ctx: &LintContext<'_>) -> Vec<Placement> {
    ctx.circuit
        .raw_elements
        .iter()
        .filter(|elem| elem.get("type").and_then(|v| v.as_str()) == Some("schematic_component"))
        .filter_map(|elem| {
            let (x, y) = (extract_coord(elem, "x")?, extract_coord(elem, "y")?);
            Some(Placement {
                refdes: resolve_refdes(elem, ctx),
                x,
                y,
            })
        })
        .collect()
}
