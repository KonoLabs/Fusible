use super::sexpr::parse_sexpr;

#[test]
fn deeply_nested_sexpr_errors_instead_of_overflowing() {
    let depth = 200_000;
    let input = format!("{}{}", "(".repeat(depth), ")".repeat(depth));
    assert!(parse_sexpr(&input).is_err());
}

#[test]
fn unicode_whitespace_separates_atoms_instead_of_hanging() {
    let parsed = parse_sexpr("(kicad_pcb\u{a0}(version\u{c}20221018))").unwrap();
    assert_eq!(format!("{parsed:?}").matches("20221018").count(), 1);
}
