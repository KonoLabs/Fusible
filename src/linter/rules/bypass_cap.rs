//! `decoupling_capacitor`: every active IC power pin must have a bypass capacitor.

use std::collections::{HashMap, HashSet};

use crate::linter::{Diagnostic, LintContext, Rule};
use crate::parser::Component;

/// `ftype` values that mark a capacitor.
const CAPACITOR_KINDS: &[&str] = &["simple_capacitor", "capacitor"];

/// `ftype` values that mark an integrated circuit.
const CHIP_KINDS: &[&str] = &["simple_chip", "chip"];

/// Verifies that integrated circuits (ICs) have decoupling/bypass capacitors
/// connected to their power supply rails to ensure power integrity.
pub struct BypassCapRule;

/// Classifies a component by its `ftype`. Only a component without one falls
/// back to the designator prefix (`C1`, `U1`) of its name or id.
fn is_kind(id: &str, component: Option<&Component>, kinds: &[&str], prefix: char) -> bool {
    match component.and_then(|c| c.kind.as_deref()) {
        Some(kind) => kinds.iter().any(|k| kind.eq_ignore_ascii_case(k)),
        None => {
            let name = component.and_then(|c| c.name.as_deref()).unwrap_or(id);
            name.starts_with(prefix) || id.starts_with(prefix)
        }
    }
}

impl Rule for BypassCapRule {
    fn name(&self) -> &'static str {
        "decoupling_capacitor"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let components: HashMap<&str, &Component> = ctx
            .circuit
            .components()
            .filter_map(|c| c.component_id.as_deref().map(|id| (id, c)))
            .collect();

        let nets_with_caps: HashSet<&str> = ctx
            .netlist
            .nets
            .iter()
            .filter(|net| {
                net.endpoints.iter().any(|ep| {
                    ep.component_id.as_deref().is_some_and(|cid| {
                        is_kind(cid, components.get(cid).copied(), CAPACITOR_KINDS, 'C')
                    })
                })
            })
            .map(|net| net.id.as_str())
            .collect();

        let mut diagnostics = Vec::new();
        let mut reported_rails: HashSet<(String, String)> = HashSet::new();

        for port in ctx.circuit.ports() {
            let Some(comp_id) = port.source_component_id.as_deref() else {
                continue;
            };
            let component = components.get(comp_id).copied();
            if !is_kind(comp_id, component, CHIP_KINDS, 'U') {
                continue;
            }
            let Some(net_node) = port
                .port_id
                .as_deref()
                .and_then(|id| ctx.netlist.port_to_net.get(id))
                .and_then(|&idx| ctx.netlist.nets.get(idx))
            else {
                continue;
            };

            let comp_name = component.and_then(|c| c.name.as_deref()).unwrap_or(comp_id);
            let pin_name = port.name.as_deref().unwrap_or("");
            let net_name = net_node.display_name();

            if is_power_rail_pin(pin_name, net_name)
                && !nets_with_caps.contains(net_node.id.as_str())
                && reported_rails.insert((comp_id.to_string(), net_node.id.clone()))
            {
                diagnostics.push(Diagnostic {
                    rule: self.name().to_string(),
                    message: format!(
                        "IC \"{comp_name}\" power pin \"{pin_name}\" has no decoupling capacitor on rail \"{net_name}\""
                    ),
                    help: Some(format!(
                        "connect a local ceramic bypass capacitor (e.g. 100nF) between \"{net_name}\" and GND near \"{comp_name}\""
                    )),
                });
            }
        }

        diagnostics
    }
}

fn is_power_rail_pin(pin: &str, net: &str) -> bool {
    let p_upper = pin.to_ascii_uppercase();
    let n_upper = net.to_ascii_uppercase();

    if p_upper.contains("GND")
        || p_upper.contains("VSS")
        || n_upper.contains("GND")
        || n_upper.contains("VSS")
    {
        return false;
    }

    ["VDD", "VCC", "VBAT", "3V3", "5V", "VBUS", "VIN"]
        .iter()
        .any(|rail| p_upper.contains(rail) || n_upper.contains(rail))
}
