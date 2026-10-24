//! Pad-to-net assignments of a KiCad footprint.

use super::sexpr::Sexpr;

/// A pad number and the net KiCad ties it to; the net is empty when the pad is unconnected.
#[derive(Debug, Clone, PartialEq)]
pub struct KicadPad {
    pub number: String,
    pub net: String,
}

/// Reads a `(pad "1" ... (net 3 "GND"))` node; `None` for a pad without a number.
pub(super) fn parse_pad(list: &[Sexpr]) -> Option<KicadPad> {
    let number = list.get(1)?.as_atom().filter(|n| !n.is_empty())?;
    let net = list
        .iter()
        .filter_map(Sexpr::as_list)
        .find(|sub| sub.first().and_then(Sexpr::as_atom) == Some("net"))
        .and_then(|sub| sub.last()?.as_atom())
        .unwrap_or_default();
    Some(KicadPad {
        number: number.to_string(),
        net: net.to_string(),
    })
}
