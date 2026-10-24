//! `bom.yml`: the prices, picks, and manual parts of a project.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::BomError;

const FILE_NAME: &str = "bom.yml";
const MAX_QUANTITY: u32 = 1_000_000;
const DEFAULT_PRICE_PER_KG: f64 = 25.0;
const DEFAULT_CURRENCY: &str = "EUR";

/// Where `bom.yml` lives for the manifest at `manifest`.
pub fn bom_path(manifest: &Path) -> PathBuf {
    manifest.with_file_name(FILE_NAME)
}

/// Print cost settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PrintSettings {
    /// Filament price per kilogram, in the BOM currency.
    pub price_per_kg: f64,
}

impl Default for PrintSettings {
    fn default() -> Self {
        Self {
            price_per_kg: DEFAULT_PRICE_PER_KG,
        }
    }
}

/// Fields set on a part. A key matching a detected part overrides it; any other key is a manual part.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supplier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mpn: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub picked: Option<bool>,
}

/// Print data of one body, keyed by the body name.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrintEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grams: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub picked: Option<bool>,
}

/// The whole `bom.yml`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BomFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(default)]
    pub print: PrintSettings,
    #[serde(default)]
    pub parts: BTreeMap<String, PartEntry>,
    #[serde(default)]
    pub prints: BTreeMap<String, PrintEntry>,
}

fn money(label: &str, value: Option<f64>) -> Result<(), BomError> {
    match value {
        Some(v) if !v.is_finite() || v < 0.0 => Err(BomError::Invalid(format!(
            "{label} must be a number of zero or more"
        ))),
        _ => Ok(()),
    }
}

fn count(label: &str, value: Option<u32>) -> Result<(), BomError> {
    match value {
        Some(q) if q > MAX_QUANTITY => Err(BomError::Invalid(format!(
            "{label} must be at most {MAX_QUANTITY}"
        ))),
        _ => Ok(()),
    }
}

impl BomFile {
    /// Reads `path`; a missing file is an empty BOM.
    pub fn load(path: &Path) -> Result<Self, BomError> {
        let path_name = path.display().to_string();
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => {
                return Err(BomError::Io {
                    path: path_name,
                    source,
                })
            }
        };
        let file: Self = serde_yaml::from_str(&text).map_err(|err| BomError::Parse {
            path: path_name,
            message: err.to_string(),
        })?;
        file.validate()?;
        Ok(file)
    }

    /// Reads `path`, writing a starter file first when none exists yet.
    pub fn load_or_create(path: &Path) -> Result<Self, BomError> {
        if !path.exists() {
            let starter = Self {
                currency: Some(DEFAULT_CURRENCY.into()),
                ..Self::default()
            };
            starter.save(path)?;
        }
        Self::load(path)
    }

    /// Writes the file through a temporary sibling so a crash never leaves half a BOM.
    pub fn save(&self, path: &Path) -> Result<(), BomError> {
        self.validate()?;
        let text = serde_yaml::to_string(self).map_err(|e| BomError::Invalid(e.to_string()))?;
        let io = |source| BomError::Io {
            path: path.display().to_string(),
            source,
        };
        let temp = path.with_extension("yml.tmp");
        std::fs::write(&temp, text).map_err(io)?;
        std::fs::rename(&temp, path).map_err(io)
    }

    /// Rejects negative or non-finite prices and absurd quantities.
    pub fn validate(&self) -> Result<(), BomError> {
        money("print.price_per_kg", Some(self.print.price_per_kg))?;
        for (id, part) in &self.parts {
            money(&format!("parts.{id}.unit_price"), part.unit_price)?;
            count(&format!("parts.{id}.quantity"), part.quantity)?;
        }
        for (id, print) in &self.prints {
            money(&format!("prints.{id}.grams"), print.grams)?;
            count(&format!("prints.{id}.quantity"), print.quantity)?;
        }
        Ok(())
    }
}
