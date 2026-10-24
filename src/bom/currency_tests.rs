use super::tests::{file, CIRCUIT};
use super::*;

#[test]
fn estimates_follow_the_currency_rate_but_user_prices_never_convert() {
    let eur = build(detect(CIRCUIT, None).unwrap(), &[], &BomFile::default());
    let usd_file = file(
        "currency: USD\nparts:\n  resistor-10kohm: {unit_price: 0.1}\nprint: {price_per_kg: 30}\n",
    );
    let usd = build(detect(CIRCUIT, None).unwrap(), &[], &usd_file);
    let rate = rate_from_eur("USD").unwrap();
    let chip = |r: &Report| {
        r.lines
            .iter()
            .find(|l| l.id == "chip-esp32-s3")
            .unwrap()
            .unit_price
    };
    assert_eq!(chip(&usd).unwrap(), chip(&eur).unwrap() * rate);
    let resistor = usd
        .lines
        .iter()
        .find(|l| l.id == "resistor-10kohm")
        .unwrap();
    assert_eq!(resistor.unit_price, Some(0.1));
    assert_eq!(usd.price_per_kg, 30.0);
    assert_eq!(usd.estimate_currency, "USD");
}

#[test]
fn an_unknown_currency_keeps_euro_estimates_and_says_so() {
    let eur = build(detect(CIRCUIT, None).unwrap(), &[], &BomFile::default());
    let odd = build(
        detect(CIRCUIT, None).unwrap(),
        &[],
        &file("currency: XYZ\n"),
    );
    assert_eq!(odd.lines, eur.lines);
    assert_eq!(odd.estimate_currency, "EUR");
    assert!(table(&odd).contains("estimates are in EUR: no rate for XYZ"));
}
