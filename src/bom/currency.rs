//! The currencies the studio offers, and the fixed rates that turn euro estimates into them.

/// Currency codes with their rate in units per euro, updated by hand once a year.
pub const CURRENCIES: [(&str, f64); 12] = [
    ("EUR", 1.0),
    ("USD", 1.08),
    ("GBP", 0.85),
    ("CHF", 0.95),
    ("CAD", 1.47),
    ("AUD", 1.65),
    ("JPY", 162.0),
    ("CNY", 7.8),
    ("SEK", 11.5),
    ("NOK", 11.6),
    ("DKK", 7.46),
    ("PLN", 4.3),
];

/// Units of `code` per euro, or `None` for a currency outside [`CURRENCIES`].
pub fn rate_from_eur(code: &str) -> Option<f64> {
    CURRENCIES
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(code))
        .map(|(_, rate)| *rate)
}

/// The canonical upper-case code for `code`, or `None` when the studio does not offer it.
pub fn normalize(code: &str) -> Option<&'static str> {
    CURRENCIES
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(code))
        .map(|(known, _)| *known)
}
