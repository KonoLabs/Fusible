use std::path::Path;

use serde::Deserialize;
use serde_yaml::Value;

pub const MANIFEST_FILENAMES: &[&str] = &["fusible.yml", "fusible.yaml"];

/// The few manifest fields the Projects page needs.
#[derive(Debug, Default, PartialEq)]
pub struct Summary {
    pub name: Option<String>,
    pub boards: usize,
    pub firmware: usize,
    pub bodies: usize,
}

#[derive(Deserialize, Default)]
struct Raw {
    name: Option<String>,
    boards: Option<Value>,
    firmware: Option<Value>,
    bodies: Option<Value>,
}

fn entries(value: Option<Value>) -> usize {
    match value {
        Some(Value::Mapping(map)) => map.len(),
        _ => 0,
    }
}

/// Parses manifest text. Unknown keys are ignored and a broken file gives an empty summary.
pub fn parse(text: &str) -> Summary {
    let raw: Raw = serde_yaml::from_str(text).unwrap_or_default();
    Summary {
        name: raw.name.filter(|name| !name.trim().is_empty()),
        boards: entries(raw.boards),
        firmware: entries(raw.firmware),
        bodies: entries(raw.bodies),
    }
}

/// Returns the manifest file of a folder, `fusible.yml` first.
pub fn find(dir: &Path) -> Option<std::path::PathBuf> {
    MANIFEST_FILENAMES
        .iter()
        .map(|file| dir.join(file))
        .find(|candidate| candidate.is_file())
}

/// Reads and summarizes the manifest of a folder, or `None` when it has none.
pub fn read(dir: &Path) -> Option<Summary> {
    let text = std::fs::read_to_string(find(dir)?).unwrap_or_default();
    Some(parse(&text))
}
