//! `decoupling_capacitor`: every active IC power pin must have a bypass capacitor.

use std::collections::{HashMap, HashSet};

use crate::linter::{Diagnostic, LintContext, Rule};

/// Verifies that integrated circuits (ICs) have decoupling/bypass capacitors
/// connected to their power supply rails to ensure power integrity.
pub struct BypassCapRule;

impl Rule for BypassCapRule {
    fn name(&self) -> &'static str {
        "decoupling_capacitor"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // 1. Map components by ID to their kind and designator
        let comp_kinds: HashMap<&str, (&str, &str)> = ctx
            .circuit
            .components()
            .filter_map(|c| {
                let id = c.component_id.as_deref()?;
                let name = c.name.as_deref().unwrap_or(id);
                let kind = c.kind.as_deref().unwrap_or("");
                Some((id, (name, kind)))
            })
            .collect();

        // 2. Identify all nets that have at least one capacitor attached
        let mut nets_with_caps: HashSet<&str> = HashSet::new();
        for net in &ctx.netlist.nets {
            let has_cap = net.endpoints.iter().any(|ep| {
                if let Some(cid) = ep.component_id.as_deref() {
                    if let Some(&(name, kind)) = comp_kinds.get(cid) {
                        return kind.eq_ignore_ascii_case("capacitor")
                            || name.starts_with('C')
                            || cid.starts_with('C');
                    }
                }
                false
            });
            if has_cap {
                nets_with_caps.insert(&net.id);
            }
        }

        // 3. Inspect every IC/chip power port
        let mut reported_rails: HashSet<(String, String)> = HashSet::new();

        for port in ctx.circuit.ports() {
            let comp_id = match port.source_component_id.as_deref() {
                Some(id) => id,
                None => continue,
            };

            let (comp_name, comp_kind) = match comp_kinds.get(comp_id) {
                Some(&(n, k)) => (n, k),
                None => (comp_id, ""),
            };

            let is_chip = comp_kind.eq_ignore_ascii_case("chip")
                || comp_kind.eq_ignore_ascii_case("simple_chip")
                || comp_name.starts_with('U')
                || comp_id.starts_with('U');

            if !is_chip {
                continue;
            }

            let pin_name = port.name.as_deref().unwrap_or("");
            let net_id = match port.net_id.as_deref() {
                Some(id) => id,
                None => continue,
            };

            let net_node = match ctx
                .netlist
                .port_to_net
                .get(port.port_id.as_deref().unwrap_or(""))
                .and_then(|&idx| ctx.netlist.nets.get(idx))
            {
                Some(node) => node,
                None => continue,
            };

            let net_name = net_node.display_name();

            // Check if this pin/net is a positive power supply rail
            if is_power_rail_pin(pin_name, net_name)
                && !nets_with_caps.contains(net_id)
                && !nets_with_caps.contains(net_node.id())
                && reported_rails.insert((comp_name.to_string(), net_name.to_string()))
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

    // Exclude ground rails
    if p_upper.contains("GND")
        || p_upper.contains("VSS")
        || n_upper.contains("GND")
        || n_upper.contains("VSS")
    {
        return false;
    }

    p_upper.contains("VDD")
        || p_upper.contains("VCC")
        || p_upper.contains("VBAT")
        || p_upper.contains("3V3")
        || p_upper.contains("5V")
        || p_upper.contains("VBUS")
        || p_upper.contains("VIN")
        || n_upper.contains("VDD")
        || n_upper.contains("VCC")
        || n_upper.contains("VBAT")
        || n_upper.contains("3V3")
        || n_upper.contains("5V")
        || n_upper.contains("VBUS")
        || n_upper.contains("VIN")
}
