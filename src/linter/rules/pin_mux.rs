//! `pin_mux`: validates MCU pin assignments against physical silicon Alternate Functions.

use crate::linter::{Diagnostic, LintContext, Rule};
use crate::silicon::{
    find_target, model::is_gpio_designator, validate_pin_assignments, AssignedPin, PinConflict,
    SiliconTarget,
};

/// Checks that assigned MCU pins are electrically supported by the target
/// microcontroller's Alternate Function (AF) matrix, strapping pins, and memory bus limits.
pub struct PinMuxRule;

impl Rule for PinMuxRule {
    fn name(&self) -> &'static str {
        "pin_mux"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let target = match ctx.mcu {
            Some(chip) => match find_target(chip) {
                Some(t) => t,
                None => return vec![unknown_mcu(chip)],
            },
            None => match detect_mcu_chip(ctx.circuit) {
                Some(t) => t,
                None => return Vec::new(),
            },
        };

        let assignments = extract_assigned_pins(ctx, target, target.name);
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
                PinConflict::UnknownPin {
                    pin_name,
                    chip_name,
                } => Diagnostic {
                    rule: self.name().to_string(),
                    message: format!(
                        "pin \"{pin_name}\" does not exist in the {chip_name} silicon database"
                    ),
                    help: Some(
                        "rename the port to a pin that exists on this chip, or add the pin to the silicon database"
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

/// Diagnostic for an explicitly configured chip the silicon database does not know.
fn unknown_mcu(chip: &str) -> Diagnostic {
    Diagnostic {
        rule: "unknown_mcu".to_string(),
        message: format!(
            "mcu \"{chip}\" is not in the silicon database; pin assignments cannot be verified"
        ),
        help: Some("use a supported chip id: esp32-s3, rp2040, stm32g431".to_string()),
    }
}

/// Finds a component whose name or kind is an exact known chip id.
fn detect_mcu_chip(circuit: &crate::parser::Circuit) -> Option<&'static SiliconTarget> {
    circuit.components().find_map(|c| {
        [c.name.as_deref(), c.kind.as_deref()]
            .into_iter()
            .flatten()
            .find_map(find_target)
    })
}

fn extract_assigned_pins(
    ctx: &LintContext<'_>,
    target: &SiliconTarget,
    chip_name: &str,
) -> Vec<AssignedPin> {
    let mut assignments = Vec::new();
    let mcu_comp = crate::codegen::pins::select_mcu_component(ctx.circuit, Some(chip_name));
    let mcu_id = mcu_comp
        .and_then(|c| c.component_id.as_deref())
        .unwrap_or("U1");
    let mcu_name = mcu_comp.and_then(|c| c.name.as_deref()).unwrap_or(mcu_id);

    for port in ctx.circuit.ports() {
        if mcu_comp.is_some() {
            let is_mcu_port = port.source_component_id.as_deref() == Some(mcu_id)
                || port.source_component_id.as_deref() == Some(mcu_name)
                || port
                    .port_id
                    .as_deref()
                    .map(|pid| pid.starts_with(mcu_id) || pid.starts_with(mcu_name))
                    .unwrap_or(false);
            if !is_mcu_port {
                continue;
            }
        }

        let pin_name = match port.name.as_deref().or_else(|| {
            port.port_hints
                .as_ref()
                .and_then(|h| h.first().map(String::as_str))
        }) {
            Some(n) => n,
            None => continue,
        };

        if target.find_pin(pin_name).is_none() && !is_gpio_designator(pin_name) {
            continue;
        }

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

        let net_upper = net_name.to_ascii_uppercase();
        let is_pulled_to_gnd = net_upper.contains("GND");
        let is_pulled_high = ["3V3", "VCC", "VDD", "VBAT", "VBUS", "5V"]
            .iter()
            .any(|rail| net_upper.contains(rail));

        assignments.push(AssignedPin {
            pin_name: pin_name.to_string(),
            net_name,
            signal_name,
            is_pulled_to_gnd,
            is_pulled_high,
        });
    }

    assignments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netlist::Netlist;
    use crate::parser::{Circuit, CircuitElement, Component, Port};

    fn esp_circuit(pin: &str) -> Circuit {
        Circuit {
            elements: vec![
                CircuitElement::Component(Component {
                    component_id: Some("U1".to_string()),
                    kind: Some("chip".to_string()),
                    name: Some("ESP32-S3".to_string()),
                }),
                CircuitElement::Port(Port {
                    port_id: Some(format!("U1.{pin}")),
                    source_component_id: Some("U1".to_string()),
                    name: Some(pin.to_string()),
                    ..Default::default()
                }),
            ],
            raw_elements: Vec::new(),
        }
    }

    fn violations(circuit: &Circuit, mcu: Option<&str>) -> Vec<Diagnostic> {
        let netlist = Netlist::from_circuit(circuit);
        let ctx = LintContext::new(circuit, &netlist).with_mcu(mcu);
        PinMuxRule.check(&ctx)
    }

    #[test]
    fn explicit_unknown_mcu_blows_the_fuse() {
        let v = violations(&esp_circuit("GPIO4"), Some("stm32f103"));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule, "unknown_mcu");
        assert!(v[0].message.contains("stm32f103"));
    }

    #[test]
    fn gpio_designator_missing_from_database_is_reported() {
        let v = violations(&esp_circuit("GPIO22"), Some("esp32-s3"));
        assert_eq!(v.len(), 1);
        assert!(v[0].message.contains("GPIO22"));
    }

    #[test]
    fn non_gpio_package_pins_are_ignored() {
        assert!(violations(&esp_circuit("VDD3V3"), Some("esp32-s3")).is_empty());
    }
}
