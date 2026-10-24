//! The `--currency` option of `fusible init`.

use std::path::Path;

use super::InitError;
use crate::bom::{bom_path, normalize_currency, BomFile, CURRENCIES};

/// Accepts only the currencies the studio offers, and returns the canonical code.
pub(super) fn parse(code: &str) -> Result<&'static str, InitError> {
    normalize_currency(code).ok_or_else(|| {
        let known: Vec<&str> = CURRENCIES.iter().map(|(code, _)| *code).collect();
        InitError::new(format!(
            "unknown currency \"{code}\" (use one of {})",
            known.join(", ")
        ))
    })
}

/// Writes `bom.yml` with `currency` through the same type the studio saves with.
pub(super) fn write_bom(base_dir: &Path, currency: &str) -> Result<(), InitError> {
    let file = BomFile {
        currency: Some(currency.into()),
        ..BomFile::default()
    };
    file.save(&bom_path(&base_dir.join("fusible.yml")))
        .map_err(|e| InitError::new(e.to_string()))
}
