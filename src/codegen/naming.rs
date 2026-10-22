//! Signal identifier derivation and collision handling.

use std::collections::{BTreeMap, BTreeSet};

use super::pins::PinMapping;

/// Sorts mappings by `(signal_name, pin_number, pin_label)` and renames
/// colliding signals to `NAME_<pin_number>`, adding `_<n>` if that still collides.
pub(super) fn dedupe_signal_names(pins: &mut [PinMapping]) {
    pins.sort_by(|a, b| {
        a.signal_name
            .cmp(&b.signal_name)
            .then_with(|| a.pin_number.cmp(&b.pin_number))
            .then_with(|| a.pin_label.cmp(&b.pin_label))
    });

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for pin in pins.iter() {
        *counts.entry(pin.signal_name.clone()).or_default() += 1;
    }

    let mut used = BTreeSet::new();
    for pin in pins.iter_mut() {
        let base = if counts[&pin.signal_name] > 1 {
            format!("{}_{}", pin.signal_name, pin.pin_number)
        } else {
            pin.signal_name.clone()
        };
        pin.signal_name = unique_name(base, &mut used);
    }

    pins.sort_by(|a, b| a.signal_name.cmp(&b.signal_name));
}

/// Returns `base`, or `base_<n>` for the smallest `n >= 2` not yet in `used`,
/// and records the result.
pub(super) fn unique_name(base: String, used: &mut BTreeSet<String>) -> String {
    let mut candidate = base.clone();
    let mut n = 2;
    while used.contains(&candidate) {
        candidate = format!("{base}_{n}");
        n += 1;
    }
    used.insert(candidate.clone());
    candidate
}

pub(super) fn derive_signal_name(
    net_name: &str,
    pin_label: &str,
    remote_comp: Option<&str>,
    remote_port: Option<&str>,
) -> String {
    let net_clean = sanitize_ident(net_name);
    if !net_clean.starts_with("TRACE_NET")
        && !net_clean.starts_with("UNNAMED")
        && !net_clean.starts_with("SOURCE_NET")
        && !net_clean.starts_with("NET")
        && !net_name.contains('>')
    {
        return net_clean;
    }

    if let (Some(comp), Some(port)) = (remote_comp, remote_port) {
        return remote_signal_name(comp, port);
    }

    if let Some((_, dest)) = net_name.split_once(" to ") {
        let trimmed = dest.trim().trim_start_matches('.');
        if let Some((comp, port)) = trimmed.split_once(" > .") {
            return remote_signal_name(comp, port);
        }
    }

    sanitize_ident(pin_label)
}

/// Builds `COMP_PORT` from a remote designator with its trailing index
/// stripped (`CAM1`, `D0` -> `CAM_D0`).
fn remote_signal_name(comp: &str, port: &str) -> String {
    let comp_clean = sanitize_ident(comp);
    let comp_prefix = comp_clean.trim_end_matches(|c: char| c.is_ascii_digit());
    sanitize_ident(&format!("{comp_prefix}_{port}"))
}

/// Converts arbitrary text to an `UPPER_SNAKE` Rust identifier. Names that
/// would start with a digit get a `PIN_` prefix; empty input becomes `PIN`.
pub(super) fn sanitize_ident(s: &str) -> String {
    let mut out = String::new();
    let mut prev_underscore = false;

    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
            prev_underscore = false;
        } else if !prev_underscore && !out.is_empty() {
            out.push('_');
            prev_underscore = true;
        }
    }

    while out.ends_with('_') {
        out.pop();
    }

    if out.is_empty() {
        "PIN".to_string()
    } else if out.starts_with(|c: char| c.is_ascii_digit()) {
        format!("PIN_{out}")
    } else {
        out
    }
}
