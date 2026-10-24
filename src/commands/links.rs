//! Cross-board checks: loads the boards named by `links:` and runs `harness_mismatch`.

use std::collections::BTreeMap;

use crate::commands::check::{load_circuit_source, CheckError};
use crate::commands::harness::link_violations;
use crate::config::FusibleConfig;
use crate::linter::FuseBlown;
use crate::parser::{self, Circuit};

fn split(end: &str) -> &str {
    end.split_once('.').map_or(end, |(board, _)| board)
}

fn load(cfg: &FusibleConfig, board: &str) -> Result<Circuit, CheckError> {
    let config = &cfg.boards[board];
    let entry = config.entry.as_ref().or(config.circuit_json.as_ref());
    let entry = entry.ok_or(CheckError::Target(
        crate::commands::target::TargetError::Missing,
    ))?;
    let (path, source) = load_circuit_source(entry)?;
    Ok(parser::parse(&source, &path)?)
}

/// Checks every link of the manifest; returns the blown fuse when a wire does not match.
pub fn check_links(cfg: &FusibleConfig) -> Result<(), CheckError> {
    let mut circuits: BTreeMap<String, Circuit> = BTreeMap::new();
    let mut violations = Vec::new();
    for (name, link) in &cfg.links {
        for end in [&link.from, &link.to] {
            let board = split(end);
            if !circuits.contains_key(board) {
                circuits.insert(board.to_string(), load(cfg, board)?);
            }
        }
        let (from, to) = (&circuits[split(&link.from)], &circuits[split(&link.to)]);
        violations.extend(link_violations(name, link, from, to));
    }
    if violations.is_empty() {
        return Ok(());
    }
    Err(CheckError::FuseBlown(FuseBlown::new(violations)))
}
