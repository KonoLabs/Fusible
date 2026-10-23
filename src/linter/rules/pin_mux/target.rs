//! Silicon target resolution for the `pin_mux` rule.

use crate::linter::{Diagnostic, LintContext};
use crate::parser::Circuit;
use crate::silicon::{find_target, SiliconTarget};

/// The configured target, else one detected from the circuit. An explicit
/// chip missing from the silicon database is an error diagnostic.
pub(super) fn resolve_target(
    ctx: &LintContext<'_>,
) -> Result<Option<&'static SiliconTarget>, Diagnostic> {
    match ctx.mcu {
        Some(chip) => find_target(chip).map(Some).ok_or_else(|| unknown_mcu(chip)),
        None => Ok(detect_mcu_chip(ctx.circuit)),
    }
}

/// Diagnostic for an explicitly configured chip the silicon database does not know.
fn unknown_mcu(chip: &str) -> Diagnostic {
    Diagnostic {
        rule: "unknown_mcu".to_string(),
        message: format!(
            "mcu \"{chip}\" is not in the silicon database; pin assignments cannot be verified"
        ),
        help: Some("use a supported chip id: esp32-s3, rp2040, stm32g431".to_string()),
        ..Default::default()
    }
}

/// Finds a component whose name or kind is an exact known chip id.
fn detect_mcu_chip(circuit: &Circuit) -> Option<&'static SiliconTarget> {
    circuit.components().find_map(|c| {
        [c.name.as_deref(), c.kind.as_deref()]
            .into_iter()
            .flatten()
            .find_map(find_target)
    })
}
