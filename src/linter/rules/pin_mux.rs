//! `pin_mux`: validates MCU pin assignments against physical silicon Alternate Functions.

use crate::linter::{Diagnostic, LintContext, Rule};
use crate::silicon::{find_target, validate_pin_assignments, AssignedPin, PinConflict};

/// Checks that assigned MCU pins are electrically supported by the target
/// microcontroller's Alternate Function (AF) matrix, strapping pins, and memory bus limits.
pub struct PinMuxRule;

impl Rule for PinMuxRule {
    fn name(&self) -> &'static str {
        "pin_mux"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let chip_name = match ctx.mcu {
            Some(m) => m,
            None => {
                // Try detecting MCU name from circuit components
                match detect_mcu_chip(ctx.circuit) {
                    Some(m) => m,
                    None => return Vec::new(),
                }
            }
        };

        let target = match find_target(chip_name) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let assignments = extract_assigned_pins(ctx);
        if assignments.is_empty() {
            return Vec::new();
        }

        let conflicts = validate_pin_assignments(target, &assignments);
        conflicts
            .into_iter()
            .map(|conflict| match conflict {
                PinConflict::UnsupportedFunction {
                    pin_name,
                    signal,
                    chip_name,
                    valid_alternatives,
                } => {
                    let sig_name = signal.name();
                    let alts_str = if valid_alternatives.is_empty() {
                        "no alternative pins support this peripheral function on this package"
                            .to_string()
                    } else {
                        let names: Vec<_> = valid_alternatives.iter().map(|p| p.name).collect();
                        format!("move {sig_name} to one of: {}", names.join(", "))
                    };

                    Diagnostic {
                        rule: self.name().to_string(),
                        message: format!(
                            "pin \"{pin_name}\" does not support peripheral function \"{sig_name}\" on {chip_name}"
                        ),
                        help: Some(alts_str),
                    }
                }
                PinConflict::PinCollision { pin_name, signals } => Diagnostic {
                    rule: self.name().to_string(),
                    message: format!(
                        "pin \"{pin_name}\" assigned to multiple conflicting peripheral functions: {}",
                        signals.join(", ")
                    ),
                    help: Some(
                        "each physical pin can only be assigned to a single peripheral function"
                            .to_string(),
                    ),
                },
                PinConflict::ReservedPinUsed { pin_name, reason } => Diagnostic {
                    rule: self.name().to_string(),
                    message: format!("pin \"{pin_name}\" is reserved: {reason}"),
                    help: Some(
                        "relocate connection away from internal memory bus pins".to_string(),
                    ),
                },
                PinConflict::StrappingConflict {
                    pin_name,
                    requirement,
                    net_name,
                } => {
                    let reason = match requirement {
                        crate::silicon::StrappingRequirement::MustBeHigh {
                            violation_consequence,
                            ..
                        } => violation_consequence,
                        crate::silicon::StrappingRequirement::MustBeLow {
                            violation_consequence,
                            ..
                        } => violation_consequence,
                    };
                    Diagnostic {
                        rule: self.name().to_string(),
                        message: format!(
                            "strapping pin \"{pin_name}\" has conflicting electrical load on net \"{net_name}\""
                        ),
                        help: Some(format!(
                            "{reason}. Ensure external circuitry does not pull this pin during reset."
                        )),
                    }
                }
            })
            .collect()
    }
}

fn detect_mcu_chip(circuit: &crate::parser::Circuit) -> Option<&'static str> {
    for c in circuit.components() {
        let name = c.name.as_deref().unwrap_or("").to_ascii_lowercase();
        let kind = c.kind.as_deref().unwrap_or("").to_ascii_lowercase();
        if name.contains("stm32") || kind.contains("stm32") {
            return Some("stm32g431");
        }
        if name.contains("rp2040") || kind.contains("rp2040") {
            return Some("rp2040");
        }
        if name.contains("esp32") || kind.contains("esp32") {
            return Some("esp32-s3");
        }
    }
    None
}

fn extract_assigned_pins(ctx: &LintContext<'_>) -> Vec<AssignedPin> {
    let mut assignments = Vec::new();

    for port in ctx.circuit.ports() {
        let pin_name = match port.name.as_deref().or_else(|| {
            port.port_hints
                .as_ref()
                .and_then(|h| h.first().map(String::as_str))
        }) {
            Some(n) => n,
            None => continue,
        };

        let port_id = match port.port_id.as_deref() {
            Some(id) => id,
            None => continue,
        };

        let net_node = ctx
            .netlist
            .port_to_net
            .get(port_id)
            .and_then(|&idx| ctx.netlist.nets.get(idx));

        let net_name = net_node
            .map(|n| n.display_name().to_string())
            .unwrap_or_else(|| format!("unconnected_{pin_name}"));

        let signal_name = net_node
            .and_then(|n| {
                n.endpoints.iter().find_map(|ep| {
                    if ep.id.as_deref() != Some(port_id) {
                        ep.name.clone()
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| net_name.clone());

        let is_pulled_to_gnd = net_name.to_ascii_uppercase().contains("GND");

        assignments.push(AssignedPin {
            pin_name: pin_name.to_string(),
            net_name,
            signal_name,
            is_pulled_to_gnd,
        });
    }

    assignments
}
