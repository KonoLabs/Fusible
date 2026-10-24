use serde::Deserialize;

const PREFIX: &str = "FUSIBLE_READY ";

/// The payload of the `FUSIBLE_READY` line.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Ready {
    pub url: String,
    pub port: u16,
}

/// Parses one stdout line, returning `None` for anything that is not the ready line.
pub fn parse(line: &str) -> Option<Ready> {
    let json = line.trim().strip_prefix(PREFIX)?;
    serde_json::from_str(json).ok()
}
