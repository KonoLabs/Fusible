//! `shorted_nets`: a trace must not tie differently named nets together.

use crate::linter::{Diagnostic, LintContext, Rule};

/// Differently named declared nets merged by one trace: GND wired to 3V3
/// describes a dead short, however the netlist resolves it.
pub struct ShortedNetsRule;

impl Rule for ShortedNetsRule {
    fn name(&self) -> &'static str {
        "shorted_nets"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        ctx.netlist
            .shorts()
            .iter()
            .map(|short| Diagnostic {
                rule: self.name().to_string(),
                message: format!(
                    "nets {} are shorted together through {}",
                    short.nets.join(", "),
                    short.trace
                ),
                help: Some(
                    "remove the trace that joins them, or rename the nets if they are meant to be one"
                        .to_string(),
                ),
                ..Default::default()
            })
            .collect()
    }
}
