//! MCU pin assignments read from the circuit for pin-mux validation.

use crate::codegen::pins::select_mcu_component;
use crate::linter::LintContext;
use crate::netlist::NetNode;
use crate::parser::Port;
use crate::silicon::{model::is_gpio_designator, AssignedPin, SiliconTarget};

/// Rails whose name marks a net as pulled high.
const HIGH_RAILS: [&str; 6] = ["3V3", "VCC", "VDD", "VBAT", "VBUS", "5V"];

/// Every MCU port that names a database pin or GPIO designator, with its net
/// and the signal name of the first other endpoint on that net.
pub(super) fn extract_assigned_pins(
    ctx: &LintContext<'_>,
    target: &SiliconTarget,
    chip_name: &str,
) -> Vec<AssignedPin> {
    let mcu_comp = select_mcu_component(ctx.circuit, Some(chip_name));
    let mcu_id = mcu_comp
        .and_then(|c| c.component_id.as_deref())
        .unwrap_or("U1");
    let mcu_name = mcu_comp.and_then(|c| c.name.as_deref()).unwrap_or(mcu_id);
    ctx.circuit
        .ports()
        .filter(|port| mcu_comp.is_none() || is_owned_by(port, mcu_id, mcu_name))
        .filter_map(|port| assigned_pin(ctx, target, port))
        .collect()
}

fn is_owned_by(port: &Port, mcu_id: &str, mcu_name: &str) -> bool {
    port.source_component_id.as_deref() == Some(mcu_id)
        || port.source_component_id.as_deref() == Some(mcu_name)
        || port
            .port_id
            .as_deref()
            .map(|pid| pid.starts_with(mcu_id) || pid.starts_with(mcu_name))
            .unwrap_or(false)
}

fn assigned_pin(ctx: &LintContext<'_>, target: &SiliconTarget, port: &Port) -> Option<AssignedPin> {
    let pin_name = port.name.as_deref().or_else(|| {
        port.port_hints
            .as_ref()
            .and_then(|h| h.first().map(String::as_str))
    })?;
    if target.find_pin(pin_name).is_none() && !is_gpio_designator(pin_name) {
        return None;
    }
    let port_id = port.port_id.as_deref()?;
    let net_node = ctx
        .netlist
        .port_to_net
        .get(port_id)
        .and_then(|&idx| ctx.netlist.nets.get(idx));
    let net_name = net_node
        .map(|n| n.display_name().to_string())
        .unwrap_or_else(|| format!("unconnected_{pin_name}"));
    let signal_name = net_node
        .and_then(|n| remote_endpoint_name(n, port_id))
        .unwrap_or_else(|| net_name.clone());
    Some(with_pull_flags(pin_name, net_name, signal_name))
}

/// Name of the first named endpoint on `node` other than `port_id`.
fn remote_endpoint_name(node: &NetNode, port_id: &str) -> Option<String> {
    node.endpoints
        .iter()
        .filter(|ep| ep.id.as_deref() != Some(port_id))
        .find_map(|ep| ep.name.clone())
}

fn with_pull_flags(pin_name: &str, net_name: String, signal_name: String) -> AssignedPin {
    let net_upper = net_name.to_ascii_uppercase();
    AssignedPin {
        pin_name: pin_name.to_string(),
        is_pulled_to_gnd: net_upper.contains("GND"),
        is_pulled_high: HIGH_RAILS.iter().any(|rail| net_upper.contains(rail)),
        net_name,
        signal_name,
    }
}
