//! Silicon Alternate Function (AF) pin-muxing validation engine.

use std::collections::BTreeMap;

use super::model::{PeripheralSignal, PinSpec, SiliconTarget, StrappingRequirement};

/// An electrical violation detected during silicon pin-mux validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinConflict {
    /// Pin assigned to a peripheral function that is not supported by its AF multiplexer.
    UnsupportedFunction {
        pin_name: String,
        signal: PeripheralSignal,
        chip_name: &'static str,
        valid_alternatives: Vec<&'static PinSpec>,
    },
    /// Multiple distinct peripheral signals assigned to the same physical pin.
    PinCollision {
        pin_name: String,
        signals: Vec<String>,
    },
    /// Pin reserved for internal memory bus (e.g. Octal Flash / PSRAM).
    ReservedPinUsed {
        pin_name: String,
        reason: &'static str,
    },
    /// Pin designator that does not exist in the silicon database for this chip.
    UnknownPin {
        pin_name: String,
        chip_name: &'static str,
    },
    /// Boot strapping pin with an incompatible electrical load.
    StrappingConflict {
        pin_name: String,
        requirement: StrappingRequirement,
        net_name: String,
    },
}

/// An assigned pin mapping to validate against the silicon database.
#[derive(Debug, Clone)]
pub struct AssignedPin {
    pub pin_name: String,
    pub net_name: String,
    pub signal_name: String,
    pub is_pulled_to_gnd: bool,
    pub is_pulled_high: bool,
}

/// Validates assigned MCU pins against a target microcontroller silicon definition.
pub fn validate_pin_assignments(
    target: &SiliconTarget,
    assignments: &[AssignedPin],
) -> Vec<PinConflict> {
    let mut conflicts = pin_collisions(assignments);
    for a in assignments {
        check_assignment(target, a, &mut conflicts);
    }
    conflicts
}

/// One [`PinConflict::PinCollision`] per pin carrying distinct signals, in pin order.
fn pin_collisions(assignments: &[AssignedPin]) -> Vec<PinConflict> {
    let mut pin_to_signals: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for a in assignments {
        pin_to_signals
            .entry(&a.pin_name)
            .or_default()
            .push(&a.signal_name);
    }
    pin_to_signals
        .into_iter()
        .filter_map(|(pin_name, mut signals)| {
            signals.sort();
            signals.dedup();
            (signals.len() > 1).then(|| PinConflict::PinCollision {
                pin_name: pin_name.to_string(),
                signals: signals.into_iter().map(str::to_string).collect(),
            })
        })
        .collect()
}

fn check_assignment(target: &SiliconTarget, a: &AssignedPin, conflicts: &mut Vec<PinConflict>) {
    let Some(pin_spec) = target.find_pin(&a.pin_name) else {
        conflicts.push(PinConflict::UnknownPin {
            pin_name: a.pin_name.clone(),
            chip_name: target.name,
        });
        return;
    };
    if pin_spec.capabilities.is_reserved_flash {
        conflicts.push(PinConflict::ReservedPinUsed {
            pin_name: a.pin_name.clone(),
            reason: "pin is permanently allocated to internal high-speed Flash/PSRAM bus",
        });
        return;
    }
    if let Some(req) = pin_spec.strapping.filter(|req| violates_strapping(req, a)) {
        conflicts.push(PinConflict::StrappingConflict {
            pin_name: a.pin_name.clone(),
            requirement: req,
            net_name: a.net_name.clone(),
        });
    }
    conflicts.extend(unsupported_function(target, pin_spec, a));
}

/// The AF conflict when the pin has a mux table that lacks the assigned signal.
fn unsupported_function(
    target: &SiliconTarget,
    pin_spec: &PinSpec,
    a: &AssignedPin,
) -> Option<PinConflict> {
    let sig =
        PeripheralSignal::parse(&a.signal_name).or_else(|| PeripheralSignal::parse(&a.net_name))?;
    if pin_spec.functions.is_empty() || pin_spec.supports_signal(&sig) {
        return None;
    }
    Some(PinConflict::UnsupportedFunction {
        pin_name: a.pin_name.clone(),
        valid_alternatives: target.find_valid_pins_for_signal(&sig),
        signal: sig,
        chip_name: target.name,
    })
}

/// True when the assigned load contradicts the pin's reset-time strapping level.
fn violates_strapping(req: &StrappingRequirement, a: &AssignedPin) -> bool {
    match req {
        StrappingRequirement::MustBeHigh { .. } => {
            let net_lower = a.net_name.to_ascii_lowercase();
            let sig_lower = a.signal_name.to_ascii_lowercase();
            a.is_pulled_to_gnd
                || net_lower.contains("gnd")
                || (net_lower.contains("led") && !net_lower.contains("vcc"))
                || (sig_lower.contains("led") && !sig_lower.contains("vcc"))
        }
        StrappingRequirement::MustBeLow { .. } => a.is_pulled_high,
    }
}
