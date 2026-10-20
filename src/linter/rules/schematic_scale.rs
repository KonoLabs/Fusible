//! `schematic_scale`: checks for out-of-bounds schematic coordinates.

use std::collections::HashSet;

use crate::linter::{Diagnostic, LintContext, Rule};

const MAX_CENTER_COORD_MM: f64 = 150.0;
const MAX_SPAN_MM: f64 = 300.0;

/// Verifies that schematic components are placed within standard schematic sheet
/// boundaries and alerts on millimeter/pixel coordinate scaling confusion.
pub struct SchematicScaleRule;

impl Rule for SchematicScaleRule {
    fn name(&self) -> &'static str {
        "schematic_scale"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut components = Vec::new();
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for elem in &ctx.circuit.raw_elements {
            if elem.get("type").and_then(|v| v.as_str()) != Some("schematic_component") {
                continue;
            }

            let x = extract_coord(elem, "x");
            let y = extract_coord(elem, "y");

            let (x, y) = match (x, y) {
                (Some(x), Some(y)) => (x, y),
                _ => continue,
            };

            let refdes = resolve_refdes(elem, ctx);

            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);

            components.push((refdes, x, y));
        }

        if components.is_empty() {
            return Vec::new();
        }

        let has_multiple_sheets = detect_multiple_sheets(ctx);
        let span_x = max_x - min_x;
        let span_y = max_y - min_y;
        let span_exceeded = !has_multiple_sheets && (span_x > MAX_SPAN_MM || span_y > MAX_SPAN_MM);

        let mut diagnostics = Vec::new();
        let mut reported = HashSet::new();

        for (refdes, x, y) in &components {
            let exceeds_center = x.abs() > MAX_CENTER_COORD_MM || y.abs() > MAX_CENTER_COORD_MM;
            let exceeds_span = span_exceeded
                && ((span_x > MAX_SPAN_MM && (*x == min_x || *x == max_x))
                    || (span_y > MAX_SPAN_MM && (*y == min_y || *y == max_y)));

            if (exceeds_center || exceeds_span) && reported.insert(refdes.clone()) {
                diagnostics.push(Diagnostic {
                    rule: self.name().to_string(),
                    message: format!(
                        "Component '{refdes}' placed at ({x:.1}, {y:.1}) exceeds standard schematic sheet bounds"
                    ),
                    help: Some(
                        "tscircuit schematic coordinates (schX, schY) are in millimeters, not screen pixels. A typical chip is 2-4mm across. Values in the hundreds cause the schematic canvas to zoom out into microscopic scale. Rescale coordinates to single-digit millimeter values."
                            .to_string(),
                    ),
                });
            }
        }

        diagnostics
    }
}

fn extract_coord(elem: &serde_json::Value, axis: &str) -> Option<f64> {
    if let Some(center) = elem.get("center") {
        if let Some(val) = center.get(axis) {
            if let Some(num) = val.as_f64() {
                return Some(num);
            }
            if let Some(str_val) = val.as_str() {
                if let Ok(parsed) = str_val.parse::<f64>() {
                    return Some(parsed);
                }
            }
        }
    }

    if let Some(val) = elem.get(axis) {
        if let Some(num) = val.as_f64() {
            return Some(num);
        }
        if let Some(str_val) = val.as_str() {
            if let Ok(parsed) = str_val.parse::<f64>() {
                return Some(parsed);
            }
        }
    }

    None
}

fn resolve_refdes(elem: &serde_json::Value, ctx: &LintContext<'_>) -> String {
    if let Some(refdes) = elem.get("refdes").and_then(|v| v.as_str()) {
        if !refdes.is_empty() {
            return refdes.to_string();
        }
    }

    if let Some(name) = elem.get("name").and_then(|v| v.as_str()) {
        if !name.is_empty() {
            return name.to_string();
        }
    }

    let src_id = elem
        .get("source_component_id")
        .and_then(|v| v.as_str())
        .or_else(|| elem.get("sourceComponentId").and_then(|v| v.as_str()));

    if let Some(id) = src_id {
        if let Some(comp) = ctx
            .circuit
            .components()
            .find(|c| c.component_id.as_deref() == Some(id))
        {
            if let Some(name) = comp.name.as_deref() {
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }

        for raw in &ctx.circuit.raw_elements {
            if raw.get("type").and_then(|v| v.as_str()) == Some("source_component") {
                let match_id = raw
                    .get("source_component_id")
                    .or_else(|| raw.get("sourceComponentId"))
                    .or_else(|| raw.get("component_id"))
                    .and_then(|v| v.as_str());

                if match_id == Some(id) {
                    if let Some(name) = raw.get("name").and_then(|v| v.as_str()) {
                        if !name.is_empty() {
                            return name.to_string();
                        }
                    }
                }
            }
        }

        return id.to_string();
    }

    if let Some(sch_id) = elem
        .get("schematic_component_id")
        .or_else(|| elem.get("schematicComponentId"))
        .and_then(|v| v.as_str())
    {
        return sch_id.to_string();
    }

    "unknown".to_string()
}

fn detect_multiple_sheets(ctx: &LintContext<'_>) -> bool {
    let sheet_count = ctx
        .circuit
        .raw_elements
        .iter()
        .filter(|e| e.get("type").and_then(|v| v.as_str()) == Some("schematic_sheet"))
        .count();

    if sheet_count > 1 {
        return true;
    }

    let distinct_sheets: HashSet<&str> = ctx
        .circuit
        .raw_elements
        .iter()
        .filter_map(|e| {
            e.get("schematic_sheet_id")
                .or_else(|| e.get("schematicSheetId"))
                .and_then(|v| v.as_str())
        })
        .collect();

    distinct_sheets.len() > 1
}
