//! `unconnected_net`: a net must not be left floating.

use crate::linter::{Diagnostic, LintContext, Rule};

/// Minimum number of endpoints for a net to count as connected.
const MIN_CONNECTIONS: usize = 2;

/// A net tied to fewer than two pads/ports is a routing omission: on the
/// real board that signal goes nowhere. The fuse blows before the bug can
/// reach the firmware.
pub struct UnconnectedNetRule;

impl Rule for UnconnectedNetRule {
    fn name(&self) -> &'static str {
        "unconnected_net"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        ctx.netlist
            .floating_nets()
            .filter(|net| net.is_defined)
            .map(|net| Diagnostic {
                rule: self.name().to_string(),
                message: format!(
                    "net \"{}\" has only {} connection(s) (minimum {MIN_CONNECTIONS}): it is floating",
                    net.display_name(),
                    net.endpoints.len()
                ),
                help: Some(
                    "every net must connect at least two pads/ports — complete the routing in tscircuit"
                        .to_string(),
                ),
                ..Default::default()
            })
            .collect()
    }
}
