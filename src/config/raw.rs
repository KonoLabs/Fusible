//! The manifest as written on disk, and its normalization into named sections.
//!
//! The single-board form (`hardware:`, `mcu:`, one `firmware:`, `mechanical:`) is
//! still accepted and becomes one board named `main`, one firmware named `firmware`,
//! and one generated body named after the project. That body keeps writing
//! `dist/mechanical/<project>.<format>`, so existing consumers of those files still work.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::types::{BoardConfig, BodyConfig, EnclosureConfig, FirmwareConfig};
use super::FusibleConfig;

/// Board name given to the legacy `hardware:` section.
pub const LEGACY_BOARD: &str = "main";
const LEGACY_FIRMWARE: &str = "firmware";
const LEGACY_BODY: &str = "enclosure";
const LEGACY_BODY_DIR: &str = "dist/mechanical";

#[derive(Debug, Deserialize, Default)]
pub(super) struct RawConfig {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    boards: BTreeMap<String, BoardConfig>,
    #[serde(default)]
    firmware: Option<FirmwareSection>,
    #[serde(default)]
    bodies: BTreeMap<String, BodyConfig>,
    #[serde(default)]
    hardware: Option<BoardConfig>,
    #[serde(default)]
    mcu: Option<LegacyMcu>,
    #[serde(default)]
    mechanical: Option<EnclosureConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FirmwareSection {
    Single(FirmwareConfig),
    Named(BTreeMap<String, FirmwareConfig>),
}

#[derive(Debug, Deserialize, Default)]
struct LegacyMcu {
    #[serde(default)]
    chip: Option<String>,
}

impl RawConfig {
    /// Folds the legacy single-board sections into named ones.
    pub(super) fn normalize(self) -> Result<FusibleConfig, String> {
        let firmware = match self.firmware {
            None => BTreeMap::new(),
            Some(FirmwareSection::Single(fw)) => BTreeMap::from([(LEGACY_FIRMWARE.into(), fw)]),
            Some(FirmwareSection::Named(map)) => map,
        };
        let bodies = bodies(self.bodies, self.mechanical, self.name.as_deref())?;
        Ok(FusibleConfig {
            name: self.name,
            boards: boards(self.boards, self.hardware, self.mcu)?,
            firmware,
            bodies,
        })
    }
}

fn boards(
    mut boards: BTreeMap<String, BoardConfig>,
    hardware: Option<BoardConfig>,
    mcu: Option<LegacyMcu>,
) -> Result<BTreeMap<String, BoardConfig>, String> {
    if hardware.is_none() && mcu.is_none() {
        return Ok(boards);
    }
    if !boards.is_empty() {
        return Err("use either `boards:` or the legacy `hardware:`/`mcu:` sections".into());
    }
    let mut board = hardware.unwrap_or_default();
    board.mcu = board.mcu.or(mcu.and_then(|m| m.chip));
    boards.insert(LEGACY_BOARD.into(), board);
    Ok(boards)
}

fn bodies(
    mut bodies: BTreeMap<String, BodyConfig>,
    mechanical: Option<EnclosureConfig>,
    project: Option<&str>,
) -> Result<BTreeMap<String, BodyConfig>, String> {
    let Some(mut enclosure) = mechanical else {
        return Ok(bodies);
    };
    if !bodies.is_empty() {
        return Err("use either `bodies:` or the legacy `mechanical:` section".into());
    }
    enclosure.out_dir = enclosure.out_dir.or_else(|| Some(LEGACY_BODY_DIR.into()));
    let body = BodyConfig {
        generate: Some(LEGACY_BODY.into()),
        enclosure,
        ..Default::default()
    };
    let name = project.filter(|n| ![LEGACY_BOARD, LEGACY_FIRMWARE].contains(n));
    bodies.insert(name.unwrap_or(LEGACY_BODY).into(), body);
    Ok(bodies)
}
