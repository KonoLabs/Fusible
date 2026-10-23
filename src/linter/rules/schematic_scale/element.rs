//! Field readers for raw `schematic_component` JSON elements.

use std::collections::HashSet;

use serde_json::Value;

use crate::linter::LintContext;

/// A number, or a string that parses as one.
fn number(val: &Value) -> Option<f64> {
    val.as_f64()
        .or_else(|| val.as_str().and_then(|s| s.parse::<f64>().ok()))
}

/// The `center.<axis>` coordinate, else the top-level `<axis>` one.
pub(super) fn extract_coord(elem: &Value, axis: &str) -> Option<f64> {
    elem.get("center")
        .and_then(|center| center.get(axis))
        .and_then(number)
        .or_else(|| elem.get(axis).and_then(number))
}

fn non_empty(text: Option<&str>) -> Option<&str> {
    text.filter(|t| !t.is_empty())
}

/// The component's designator: its `refdes` or `name`, else the name of its
/// source component, else its source or schematic id, else `unknown`.
pub(super) fn resolve_refdes(elem: &Value, ctx: &LintContext<'_>) -> String {
    let text = |key: &str| elem.get(key).and_then(Value::as_str);
    non_empty(text("refdes"))
        .or_else(|| non_empty(text("name")))
        .map(str::to_string)
        .or_else(|| {
            let id = text("source_component_id").or_else(|| text("sourceComponentId"))?;
            Some(source_component_name(ctx, id).unwrap_or(id).to_string())
        })
        .or_else(|| {
            elem.get("schematic_component_id")
                .or_else(|| elem.get("schematicComponentId"))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// The non-empty name of source component `id`, from the parsed components
/// first, then from raw `source_component` elements.
fn source_component_name<'a>(ctx: &'a LintContext<'_>, id: &str) -> Option<&'a str> {
    ctx.circuit
        .components()
        .find(|c| c.component_id.as_deref() == Some(id))
        .and_then(|c| non_empty(c.name.as_deref()))
        .or_else(|| raw_component_name(ctx, id))
}

fn raw_component_name<'a>(ctx: &'a LintContext<'_>, id: &str) -> Option<&'a str> {
    ctx.circuit
        .raw_elements
        .iter()
        .filter(|raw| raw.get("type").and_then(Value::as_str) == Some("source_component"))
        .filter(|raw| {
            raw.get("source_component_id")
                .or_else(|| raw.get("sourceComponentId"))
                .or_else(|| raw.get("component_id"))
                .and_then(Value::as_str)
                == Some(id)
        })
        .find_map(|raw| non_empty(raw.get("name").and_then(Value::as_str)))
}

/// True when the circuit spans more than one schematic sheet.
pub(super) fn detect_multiple_sheets(ctx: &LintContext<'_>) -> bool {
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
