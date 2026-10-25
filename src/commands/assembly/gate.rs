//! The fuse gate of the assembly: every board it needs is resolved and linted first.

use std::collections::BTreeMap;
use std::path::PathBuf;

use super::AssemblyError;
use crate::commands::check;
use crate::commands::target::{is_tsx_target, resolve_board, BoardTarget};
use crate::config::{AssemblyConfig, FusibleConfig};

/// Each needed board with the Circuit JSON that passed the gate, keyed by board name.
pub(super) type Circuits = BTreeMap<String, (BoardTarget, String)>;

/// Names of the boards whose circuit the assembly needs: board parts and the boards bodies are fitted to.
fn needed_boards<'a>(cfg: &'a FusibleConfig, assembly: &'a AssemblyConfig) -> Vec<&'a str> {
    let mut names: Vec<&str> = assembly
        .parts
        .keys()
        .filter_map(|name| match cfg.bodies.get(name) {
            None => Some(name.as_str()),
            Some(body) => cfg.body_board(body),
        })
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

/// Resolves the needed boards; with `modeled`, a board placed as a part must have a TSX entry.
fn resolve_needed<'a>(
    cfg: &'a FusibleConfig,
    assembly: &'a AssemblyConfig,
    modeled: bool,
) -> Result<Vec<(&'a str, BoardTarget)>, AssemblyError> {
    let boards = needed_boards(cfg, assembly)
        .into_iter()
        .map(|name| Ok((name, resolve_board(Some(PathBuf::from(name)))?)))
        .collect::<Result<Vec<_>, AssemblyError>>()?;
    let unmodeled = boards.iter().find(|(name, board)| {
        modeled && assembly.parts.contains_key(*name) && !is_tsx_target(&board.entry)
    });
    match unmodeled {
        Some((name, _)) => Err(AssemblyError::NoBoardModel {
            name: name.to_string(),
        }),
        None => Ok(boards),
    }
}

/// Lints every needed board unless `skip_lint` is set, before any file is written.
pub(super) fn fuse_gate(
    cfg: &FusibleConfig,
    assembly: &AssemblyConfig,
    skip_lint: bool,
) -> Result<Circuits, AssemblyError> {
    let boards = resolve_needed(cfg, assembly, true)?;
    if skip_lint {
        println!("⚠️  Skipping electrical rule verification (--skip-lint requested).\n");
    } else {
        println!("🔍 Verifying electrical rules before building the assembly (fuse check)...");
    }
    let mut circuits = BTreeMap::new();
    for (name, board) in boards {
        let source = if skip_lint {
            check::load_circuit_source(&board.entry)?.1
        } else {
            check::lint_source(&board.entry, board.mcu.clone())?.1
        };
        circuits.insert(name.to_string(), (board, source));
    }
    if !skip_lint {
        println!("⚡ Fuse intact — proceeding with the assembly.\n");
    }
    Ok(circuits)
}

/// Loads the Circuit JSON of every needed board without linting or printing anything.
pub(super) fn load_circuits(
    cfg: &FusibleConfig,
    assembly: &AssemblyConfig,
) -> Result<Circuits, AssemblyError> {
    let mut circuits = BTreeMap::new();
    for (name, board) in resolve_needed(cfg, assembly, false)? {
        let source = check::load_circuit_source(&board.entry)?.1;
        circuits.insert(name.to_string(), (board, source));
    }
    Ok(circuits)
}
