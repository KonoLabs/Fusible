//! `ghost_net`: every referenced net must be defined.

use crate::linter::{Diagnostic, LintContext, Rule};

/// A `net_id` referenced by ports, pads, or traces but defined by no `source_net`.
pub struct GhostNetRule;

impl Rule for GhostNetRule {
    fn name(&self) -> &'static str {
        "ghost_net"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        ctx.netlist
            .ghost_nets()
            .map(|net| Diagnostic {
                rule: self.name().to_string(),
                message: format!(
                    "net \"{}\" is referenced by {} connection(s) but no source_net defines it",
                    net.id,
                    net.reference_count
                ),
                help: Some(
                    "add the missing source_net element, or fix the net_id reference on the port/pad"
                        .to_string(),
                ),
            })
            .collect()
    }
}
