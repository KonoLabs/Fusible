//! The `harness_mismatch` rule: cabled connectors must carry matching nets.

use crate::config::{LinkConfig, WireConfig};
use crate::linter::Diagnostic;
use crate::netlist::Netlist;
use crate::parser::Circuit;

const RULE: &str = "harness_mismatch";

struct Side {
    circuit: Circuit,
    netlist: Netlist,
}

enum Pin {
    NoSuchPart,
    Absent,
    Floating,
    Net(String),
}

fn find_pin(side: &Side, refdes: &str, pin: &str) -> Pin {
    let Some(component) = side
        .circuit
        .components()
        .find(|c| c.name.as_deref() == Some(refdes) || c.component_id.as_deref() == Some(refdes))
    else {
        return Pin::NoSuchPart;
    };
    let owner = component.component_id.as_deref();
    let port = side.circuit.ports().find(|p| {
        let prefix = p.port_id.as_deref().and_then(|id| id.split_once('.'));
        let belongs = match &p.source_component_id {
            Some(id) => Some(id.as_str()) == owner,
            None => prefix.map(|(id, _)| id) == owner,
        };
        belongs
            && (p.pin_number.is_some_and(|n| n.to_string() == pin)
                || prefix.is_some_and(|(_, suffix)| suffix == pin)
                || p.name
                    .as_deref()
                    .is_some_and(|n| n.eq_ignore_ascii_case(pin)))
    });
    let Some(port) = port else {
        return Pin::Absent;
    };
    port.port_id
        .as_ref()
        .and_then(|id| side.netlist.port_to_net.get(id))
        .map_or(Pin::Floating, |&i| {
            Pin::Net(side.netlist.nets[i].display_name().to_string())
        })
}

fn split(end: &str) -> (&str, &str) {
    end.split_once('.').unwrap_or((end, ""))
}

fn expected(wire: &WireConfig) -> (Option<&str>, Option<&str>) {
    match (&wire.net, &wire.nets) {
        (Some(net), _) => (Some(net), Some(net)),
        (None, Some([a, b])) => (Some(a), Some(b)),
        (None, None) => (None, None),
    }
}

fn describe(end: &str, pin: &str, found: &Pin) -> Result<String, String> {
    let (board, refdes) = split(end);
    let at = format!("{board}.{refdes} pin {pin}");
    match found {
        Pin::NoSuchPart => Err(format!("{board} has no part {refdes}")),
        Pin::Absent => Err(format!("{at} does not exist")),
        Pin::Floating => Err(format!("{at} is not connected to any net")),
        Pin::Net(net) => Ok(net.clone()),
    }
}

fn check_wire(
    name: &str,
    link: &LinkConfig,
    wire: &WireConfig,
    sides: (&Side, &Side),
) -> Option<String> {
    let (a_pin, b_pin) = (wire.a.to_string(), wire.b.to_string());
    let a_net = describe(
        &link.from,
        &a_pin,
        &find_pin(sides.0, split(&link.from).1, &a_pin),
    );
    let b_net = describe(
        &link.to,
        &b_pin,
        &find_pin(sides.1, split(&link.to).1, &b_pin),
    );
    let (a_net, b_net) = match (a_net, b_net) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(e), _) | (_, Err(e)) => return Some(format!("[link {name}] {e}")),
    };
    let (want_a, want_b) = expected(wire);
    let same = |x: &str, y: &str| x.eq_ignore_ascii_case(y);
    let wrong = match (want_a, want_b) {
        (Some(wa), Some(wb)) => !same(&a_net, wa) || !same(&b_net, wb),
        _ => !same(&a_net, &b_net),
    };
    wrong.then(|| {
        format!(
            "[link {name}] {} pin {a_pin} carries {a_net} but {} pin {b_pin} carries {b_net}",
            link.from, link.to
        )
    })
}

fn violation(message: String) -> Diagnostic {
    Diagnostic {
        rule: RULE.into(),
        message,
        help: Some(
            "fix the wiring on one board, or correct the wire in `links:` of fusible.yml".into(),
        ),
        ..Default::default()
    }
}

pub(crate) fn link_violations(
    name: &str,
    link: &LinkConfig,
    from: &Circuit,
    to: &Circuit,
) -> Vec<Diagnostic> {
    let sides = (
        &Side {
            netlist: Netlist::from_circuit(from),
            circuit: from.clone(),
        },
        &Side {
            netlist: Netlist::from_circuit(to),
            circuit: to.clone(),
        },
    );
    link.wires
        .iter()
        .filter_map(|wire| check_wire(name, link, wire, sides))
        .map(violation)
        .collect()
}
