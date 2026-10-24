use super::*;

pub(super) const CIRCUIT: &str = r#"[
  {"type":"source_component","source_component_id":"sc1","name":"R1",
   "ftype":"simple_resistor","resistance":10000},
  {"type":"source_component","source_component_id":"sc2","name":"R2",
   "ftype":"simple_resistor","resistance":10000},
  {"type":"source_component","source_component_id":"sc3","name":"U1",
   "ftype":"simple_chip","manufacturer_part_number":"ESP32-S3"},
  {"type":"source_component","source_component_id":"sc4","name":"V1","ftype":"simple_power_source"},
  {"type":"pcb_component","source_component_id":"sc1"},
  {"type":"pcb_component","source_component_id":"sc2"},
  {"type":"pcb_component","source_component_id":"sc3"}
]"#;

pub(super) fn file(yaml: &str) -> BomFile {
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn identical_parts_group_and_unplaced_sources_stay_out() {
    let report = build(detect(CIRCUIT, None).unwrap(), &[], &BomFile::default());
    let ids: Vec<_> = report
        .lines
        .iter()
        .map(|l| (l.id.as_str(), l.quantity))
        .collect();
    assert_eq!(ids, [("chip-esp32-s3", 1), ("resistor-10kohm", 2)]);
}

#[test]
fn prices_picks_and_prints_feed_the_totals() {
    let yaml = "parts:\n  resistor-10kohm: {unit_price: 0.1}\n  m3-screw: {quantity: 4, unit_price: 0.25}\n  \
                chip-esp32-s3: {unit_price: 3.0, picked: false}\nprints:\n  case: {grams: 40}\n";
    let report = build(
        detect(CIRCUIT, None).unwrap(),
        &[Body {
            name: "case".into(),
            grams: None,
        }],
        &file(yaml),
    );
    assert_eq!(report.totals.electronics, 0.2);
    assert_eq!(report.totals.manual, 1.0);
    assert_eq!(report.totals.printed, 1.0);
    assert_eq!(report.totals.total, 2.2);
}

#[test]
fn boards_share_one_line_with_scoped_designators() {
    let mut parts = detect(CIRCUIT, Some("a")).unwrap();
    parts.extend(detect(CIRCUIT, Some("b")).unwrap());
    let report = build(parts, &[], &BomFile::default());
    assert_eq!(report.lines[1].quantity, 4);
    assert_eq!(report.lines[1].designators[0], "a/R1");
}

#[test]
fn a_negative_price_is_rejected() {
    let bad = file("parts:\n  x: {unit_price: -1}\n");
    assert!(bad.validate().is_err());
}

#[test]
fn csv_cells_cannot_start_a_formula() {
    let bom = file("parts:\n  evil: {description: '=cmd()'}\n");
    let text = csv(&build(Vec::new(), &[], &bom));
    assert!(text.contains(",'=cmd(),"));
}

#[test]
fn unpriced_parts_get_an_estimate_and_a_set_price_wins() {
    let parts = detect(CIRCUIT, None).unwrap();
    let report = build(parts.clone(), &[], &BomFile::default());
    assert!(report.lines.iter().all(|l| l.estimated));
    assert_eq!(report.totals.estimated, 2);
    assert_eq!(report.totals.unpriced, 0);
    let priced = build(
        parts,
        &[],
        &file("parts:\n  resistor-10kohm: {unit_price: 0.5}\n"),
    );
    let resistor = priced
        .lines
        .iter()
        .find(|l| l.id == "resistor-10kohm")
        .unwrap();
    assert!(!resistor.estimated);
    assert_eq!(resistor.unit_price, Some(0.5));
}

#[test]
fn a_measured_weight_prices_the_print_until_grams_are_typed() {
    let body = [Body {
        name: "case".into(),
        grams: Some(50.0),
    }];
    let report = build(Vec::new(), &body, &BomFile::default());
    assert!(report.lines[0].estimated);
    assert_eq!(report.totals.printed, 1.25);
    let typed = build(Vec::new(), &body, &file("prints:\n  case: {grams: 10}\n"));
    assert!(!typed.lines[0].estimated);
    assert_eq!(typed.totals.printed, 0.25);
}
