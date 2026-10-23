//! `pin_mux`: validates MCU pin assignments against physical silicon Alternate Functions.

mod assignments;
mod target;

use crate::linter::{Diagnostic, LintContext, Rule};
use crate::silicon::{
    validate_pin_assignments, PeripheralSignal, PinConflict, PinSpec, StrappingRequirement,
};
use assignments::extract_assigned_pins;
use target::resolve_target;

const COLLISION_HELP: &str =
    "each physical pin can only be assigned to a single peripheral function";
const UNKNOWN_PIN_HELP: &str =
    "rename the port to a pin that exists on this chip, or add the pin to the silicon database";
const RESERVED_HELP: &str = "relocate connection away from internal memory bus pins";

/// Checks that assigned MCU pins are electrically supported by the target
/// microcontroller's Alternate Function (AF) matrix, strapping pins, and memory bus limits.
pub struct PinMuxRule;

impl Rule for PinMuxRule {
    fn name(&self) -> &'static str {
        "pin_mux"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let target = match resolve_target(ctx) {
            Ok(Some(target)) => target,
            Ok(None) => return Vec::new(),
            Err(unknown) => return vec![unknown],
        };
        let assignments = extract_assigned_pins(ctx, target, target.name);
        validate_pin_assignments(target, &assignments)
            .iter()
            .map(|conflict| Diagnostic {
                rule: self.name().to_string(),
                message: conflict_message(conflict),
                help: Some(conflict_help(conflict)),
                ..Default::default()
            })
            .collect()
    }
}

fn conflict_message(conflict: &PinConflict) -> String {
    match conflict {
        PinConflict::UnsupportedFunction {
            pin_name,
            signal,
            chip_name,
            ..
        } => {
            let sig_name = signal.name();
            format!(
                "pin \"{pin_name}\" does not support peripheral function \"{sig_name}\" on {chip_name}"
            )
        }
        PinConflict::PinCollision { pin_name, signals } => format!(
            "pin \"{pin_name}\" assigned to multiple conflicting peripheral functions: {}",
            signals.join(", ")
        ),
        PinConflict::UnknownPin {
            pin_name,
            chip_name,
        } => format!("pin \"{pin_name}\" does not exist in the {chip_name} silicon database"),
        PinConflict::ReservedPinUsed { pin_name, reason } => {
            format!("pin \"{pin_name}\" is reserved: {reason}")
        }
        PinConflict::StrappingConflict {
            pin_name, net_name, ..
        } => format!(
            "strapping pin \"{pin_name}\" has conflicting electrical load on net \"{net_name}\""
        ),
    }
}

fn conflict_help(conflict: &PinConflict) -> String {
    match conflict {
        PinConflict::UnsupportedFunction {
            signal,
            valid_alternatives,
            ..
        } => alternatives_help(signal, valid_alternatives),
        PinConflict::PinCollision { .. } => COLLISION_HELP.to_string(),
        PinConflict::UnknownPin { .. } => UNKNOWN_PIN_HELP.to_string(),
        PinConflict::ReservedPinUsed { .. } => RESERVED_HELP.to_string(),
        PinConflict::StrappingConflict { requirement, .. } => format!(
            "{}. Ensure external circuitry does not pull this pin during reset.",
            strapping_consequence(requirement)
        ),
    }
}

fn alternatives_help(signal: &PeripheralSignal, valid_alternatives: &[&PinSpec]) -> String {
    if valid_alternatives.is_empty() {
        return "no alternative pins support this peripheral function on this package".to_string();
    }
    let names: Vec<_> = valid_alternatives.iter().map(|p| p.name).collect();
    format!("move {} to one of: {}", signal.name(), names.join(", "))
}

fn strapping_consequence(requirement: &StrappingRequirement) -> &'static str {
    match requirement {
        StrappingRequirement::MustBeHigh {
            violation_consequence,
            ..
        }
        | StrappingRequirement::MustBeLow {
            violation_consequence,
            ..
        } => violation_consequence,
    }
}
