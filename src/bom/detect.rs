//! Detects the physical parts of a Circuit JSON, grouped by kind, value, and part number.

use std::collections::HashSet;

use serde_json::{Map, Value};

use super::value::{mpn_of, value_of};
use super::BomError;

/// One component, with the id of the group it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub id: String,
    pub kind: String,
    pub value: Option<String>,
    pub mpn: Option<String>,
    pub designators: Vec<String>,
}

fn field<'a>(object: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| object.get(*key)?.as_str())
        .filter(|s| !s.is_empty())
}

fn slug(text: &str) -> String {
    let ascii = text.replace('Ω', "ohm").replace(['µ', 'μ'], "u");
    let mapped: String = ascii
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    mapped
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn part_id(kind: &str, value: Option<&str>, mpn: Option<&str>, name: &str) -> String {
    let identity = match (value, mpn) {
        (None, None) => vec![name],
        (value, mpn) => value.into_iter().chain(mpn).collect(),
    };
    slug(&format!("{kind} {}", identity.join(" ")))
}

fn mounted(elements: &[Value]) -> HashSet<&str> {
    elements
        .iter()
        .filter_map(Value::as_object)
        .filter(|o| o.get("type").and_then(Value::as_str) == Some("pcb_component"))
        .filter_map(|o| o.get("source_component_id")?.as_str())
        .collect()
}

fn describe(object: &Map<String, Value>, scope: Option<&str>) -> Option<(Part, String)> {
    let source_id = field(object, &["source_component_id", "component_id"])?;
    let name = field(object, &["name"]).unwrap_or(source_id);
    let kind = field(object, &["ftype", "kind"]).unwrap_or("component");
    let kind = kind.strip_prefix("simple_").unwrap_or(kind);
    let (value, mpn) = (value_of(object), mpn_of(object));
    let designator = scope.map_or_else(|| name.to_string(), |s| format!("{s}/{name}"));
    let id = part_id(kind, value.as_deref(), mpn.as_deref(), name);
    let part = Part {
        id,
        kind: kind.into(),
        value,
        mpn,
        designators: vec![designator],
    };
    Some((part, source_id.to_string()))
}

/// Lists the components of `source` that exist on the board, one part per component.
///
/// When the circuit has a PCB, only components placed on it count, so sources and
/// probes stay out. `scope` prefixes the designators, to tell boards apart. The report
/// groups identical parts.
pub fn detect(source: &str, scope: Option<&str>) -> Result<Vec<Part>, BomError> {
    let elements: Vec<Value> =
        serde_json::from_str(source).map_err(|e| BomError::Circuit(e.to_string()))?;
    let placed = mounted(&elements);
    let parts = elements
        .iter()
        .filter_map(Value::as_object)
        .filter(|o| o.get("type").and_then(Value::as_str) == Some("source_component"))
        .filter_map(|o| describe(o, scope))
        .filter(|(_, source_id)| placed.is_empty() || placed.contains(source_id.as_str()))
        .map(|(part, _)| part);
    Ok(parts.collect())
}
