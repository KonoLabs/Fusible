//! `dangling_reference`: every referenced port and PCB port must exist.

use crate::linter::{Diagnostic, LintContext, Rule};

/// A trace, pad, or PCB port pointing at an element nobody defines. The
/// netlist adds no endpoint for it, so without this rule the missing copper
/// would vanish silently.
pub struct DanglingReferenceRule;

impl Rule for DanglingReferenceRule {
    fn name(&self) -> &'static str {
        "dangling_reference"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        ctx.netlist
            .dangling_refs()
            .iter()
            .map(|r| Diagnostic {
                rule: self.name().to_string(),
                message: format!("{} references {}, which does not exist", r.owner, r.target),
                help: Some(
                    "add the missing element, or fix the reference: its connection cannot be verified"
                        .to_string(),
                ),
            })
            .collect()
    }
}
