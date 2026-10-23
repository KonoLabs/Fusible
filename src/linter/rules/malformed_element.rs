//! `malformed_element`: a recognized element must carry its identifier.

use crate::linter::{Diagnostic, LintContext, Rule};
use crate::parser::CircuitElement;

/// A recognized element without an identifier, or a trace that connects
/// nothing.
///
/// `Option` in the model keeps *deserialization* tolerant, but canonical
/// circuit-json always provides these identifiers: `None` means the input
/// deviates from the format. A linter that silently skips what it cannot
/// read validates malformed input — the worst failure mode for a checker.
pub struct MalformedElementRule;

/// What the element is, what it is missing, and the field that fixes it.
type ElementSummary<'a> = (&'a str, Option<&'a str>, &'a str);

/// Splits one AST element into (kind, identifier, expected field). Unmodeled
/// types carry no promise we could check, hence `None`.
fn summarize(element: &CircuitElement) -> Option<ElementSummary<'_>> {
    Some(match element {
        CircuitElement::Component(c) => (
            "source_component",
            c.component_id.as_deref(),
            "source_component_id",
        ),
        CircuitElement::Net(n) => ("source_net", n.net_id.as_deref(), "source_net_id"),
        CircuitElement::Port(p) => ("source_port", p.port_id.as_deref(), "source_port_id"),
        CircuitElement::Pad(p) => ("source_pad", p.pad_id.as_deref(), "source_pad_id"),
        CircuitElement::PcbPort(p) => ("pcb_port", p.pcb_port_id.as_deref(), "pcb_port_id"),
        CircuitElement::Trace(t) => ("source_trace", t.trace_id.as_deref(), "source_trace_id"),
        CircuitElement::Unknown => return None,
    })
}

fn missing_identifier(index: usize, kind: &str, expected: &str) -> Diagnostic {
    Diagnostic {
        rule: "malformed_element".to_string(),
        message: format!(
            "element #{index} ({kind}) has no identifier: \
             its connections cannot be verified"
        ),
        help: Some(format!(
            "add a `{expected}` field — canonical circuit-json always \
             carries it, so this file deviates from the format"
        )),
        ..Default::default()
    }
}

fn empty_trace(index: usize) -> Diagnostic {
    Diagnostic {
        rule: "malformed_element".to_string(),
        message: format!("element #{index} (source_trace) connects no port and no net"),
        help: Some(
            "fill `connected_source_port_ids` or `connected_source_net_ids`, or delete the trace"
                .to_string(),
        ),
        ..Default::default()
    }
}

/// Every defect of one element: a missing identifier, and a trace whose
/// connection lists are both empty.
fn defects(index: usize, element: &CircuitElement) -> Vec<Diagnostic> {
    let mut found = Vec::new();
    if let Some((kind, None, expected)) = summarize(element) {
        found.push(missing_identifier(index, kind, expected));
    }
    if let CircuitElement::Trace(t) = element {
        if t.connected_source_port_ids.is_empty() && t.connected_source_net_ids.is_empty() {
            found.push(empty_trace(index));
        }
    }
    found
}

impl Rule for MalformedElementRule {
    fn name(&self) -> &'static str {
        "malformed_element"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        ctx.circuit
            .elements
            .iter()
            .enumerate()
            .flat_map(|(index, element)| defects(index, element))
            .collect()
    }
}
