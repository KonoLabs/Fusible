use super::*;
use crate::config::error::ConfigError;

fn invalid(yaml: &str) -> String {
    match FusibleConfig::from_str(yaml) {
        Err(ConfigError::Invalid(message)) => message,
        other => panic!("expected ConfigError::Invalid, got {other:?}"),
    }
}

#[test]
fn parses_links_and_rejects_bad_ones() {
    let base = "boards: {a: {}, b: {}}\nlinks:\n  l:\n";
    let cfg = FusibleConfig::from_str(&format!(
        "{base}    from: a.J1\n    to: b.J2\n    wires: [{{a: 1, b: 2, net: GND}}, {{a: TX, b: RX}}]"
    ))
    .expect("valid links");
    assert_eq!(cfg.links["l"].wires.len(), 2);
    assert_eq!(cfg.links["l"].wires[1].a.to_string(), "TX");
    assert!(invalid(&format!(
        "{base}    from: a\n    to: b.J2\n    wires: [{{a: 1, b: 1}}]"
    ))
    .contains("board.REFDES"));
    assert!(invalid(&format!(
        "{base}    from: z.J1\n    to: b.J2\n    wires: [{{a: 1, b: 1}}]"
    ))
    .contains("unknown board `z`"));
    assert!(invalid(&format!(
        "{base}    from: a.J1\n    to: b.J2\n    wires: []"
    ))
    .contains("no wires"));
    assert!(invalid(&format!(
        "{base}    from: a.J1\n    to: b.J2\n    wires: [{{a: 1, b: 1, net: X, nets: [X, Y]}}]"
    ))
    .contains("both `net` and `nets`"));
}
