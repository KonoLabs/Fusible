//! `decoupling_capacitor`: every active IC power pin must have a bypass capacitor.

use std::collections::{HashMap, HashSet};

use crate::linter::{Diagnostic, LintContext, Rule};
use crate::netlist::NetNode;
use crate::parser::{Component, Port};

type ComponentIndex<'a> = HashMap<&'a str, &'a Component>;

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

/// An IC power pin and the rail net it sits on.
struct PowerPin<'a> {
    comp_id: &'a str,
    comp_name: &'a str,
    pin_name: &'a str,
    net: &'a NetNode,
}

impl Rule for BypassCapRule {
    fn name(&self) -> &'static str {
        "decoupling_capacitor"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let components: ComponentIndex<'_> = ctx
            .circuit
            .components()
            .filter_map(|c| c.component_id.as_deref().map(|id| (id, c)))
            .collect();
        let nets_with_caps = nets_with_capacitors(ctx, &components);
        let mut reported_rails: HashSet<(String, String)> = HashSet::new();
        ctx.circuit
            .ports()
            .filter_map(|port| chip_power_pin(ctx, &components, port))
            .filter(|pin| {
                !nets_with_caps.contains(pin.net.id.as_str())
                    && reported_rails.insert((pin.comp_id.to_string(), pin.net.id.clone()))
            })
            .map(|pin| self.missing_cap(&pin))
            .collect()
    }
}

impl BypassCapRule {
    fn missing_cap(&self, pin: &PowerPin<'_>) -> Diagnostic {
        let PowerPin {
            comp_name,
            pin_name,
            ..
        } = pin;
        let net_name = pin.net.display_name();
        Diagnostic {
            rule: self.name().to_string(),
            message: format!(
                "IC \"{comp_name}\" power pin \"{pin_name}\" has no decoupling capacitor \
                 on rail \"{net_name}\""
            ),
            help: Some(format!(
                "connect a local ceramic bypass capacitor (e.g. 100nF) between \"{net_name}\" \
                 and GND near \"{comp_name}\""
            )),
        }
    }
}

/// Ids of nets with at least one capacitor endpoint.
fn nets_with_capacitors<'a>(
    ctx: &'a LintContext<'_>,
    components: &ComponentIndex<'_>,
) -> HashSet<&'a str> {
    ctx.netlist
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
        .collect()
}

/// The port as an IC power pin, when its component is a chip and its net a power rail.
fn chip_power_pin<'a>(
    ctx: &'a LintContext<'_>,
    components: &ComponentIndex<'a>,
    port: &'a Port,
) -> Option<PowerPin<'a>> {
    let comp_id = port.source_component_id.as_deref()?;
    let component = components.get(comp_id).copied();
    if !is_kind(comp_id, component, CHIP_KINDS, 'U') {
        return None;
    }
    let net = port
        .port_id
        .as_deref()
        .and_then(|id| ctx.netlist.port_to_net.get(id))
        .and_then(|&idx| ctx.netlist.nets.get(idx))?;
    let pin = PowerPin {
        comp_id,
        comp_name: component.and_then(|c| c.name.as_deref()).unwrap_or(comp_id),
        pin_name: port.name.as_deref().unwrap_or(""),
        net,
    };
    is_power_rail_pin(pin.pin_name, net.display_name()).then_some(pin)
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
