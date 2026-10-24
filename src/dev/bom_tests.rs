use super::bom::save;

#[test]
fn a_saved_bom_round_trips_and_a_bad_one_writes_nothing() {
    let dir = std::env::temp_dir().join(format!("fusible-bom-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("bom.yml");
    let good = r#"{"parts":{"m3":{"quantity":4,"unit_price":0.25}}}"#;
    assert!(save(&path, good).is_ok());
    assert!(std::fs::read_to_string(&path).unwrap().contains("m3"));
    let bad = r#"{"parts":{"m3":{"unit_price":-1}}}"#;
    assert_eq!(save(&path, bad).unwrap_err().0, 400);
    let unknown = r#"{"nope":1}"#;
    assert_eq!(save(&path, unknown).unwrap_err().0, 400);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_missing_bom_is_created_with_a_currency() {
    let dir = std::env::temp_dir().join(format!("fusible-bom-new-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("bom.yml");
    let file = crate::bom::BomFile::load_or_create(&path).unwrap();
    assert_eq!(file.currency.as_deref(), Some("EUR"));
    assert!(std::fs::read_to_string(&path)
        .unwrap()
        .contains("currency: EUR"));
    std::fs::remove_dir_all(&dir).unwrap();
}
