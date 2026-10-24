use super::timestamp::rfc3339;

#[test]
fn formats_known_instants() {
    assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
    assert_eq!(rfc3339(951_782_400), "2000-02-29T00:00:00Z");
    assert_eq!(rfc3339(1_792_800_000), "2026-10-24T00:00:00Z");
    assert_eq!(rfc3339(1_792_886_399), "2026-10-24T23:59:59Z");
}
