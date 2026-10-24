//! Shared helper for resolving circuit target files from CLI or manifest.

use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::config::{BoardConfig, FusibleConfig};

mod kind;

use kind::entry_from_conventions;
pub use kind::{is_kicad_target, is_tsx_target};

/// A command target that could not be resolved to a board.
#[derive(Debug, Error, Diagnostic)]
pub enum TargetError {
    #[error("no circuit file specified and no fusible.yml found")]
    #[diagnostic(
        code(fusible::target::missing),
        help("specify a file via arguments or run `fusible init` to create a manifest")
    )]
    Missing,

    #[error("this project has several boards: {}", names.join(", "))]
    #[diagnostic(
        code(fusible::target::ambiguous),
        help("name the board to use, e.g. `fusible <command> {}`", names[0])
    )]
    Ambiguous { names: Vec<String> },
}

/// A board resolved from the CLI and the manifest, with the settings that apply to it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BoardTarget {
    /// Manifest board name, or `None` for a file outside any manifest board.
    pub name: Option<String>,
    pub entry: PathBuf,
    pub title: String,
    pub mcu: Option<String>,
    /// Names of the firmware bound to this board.
    pub firmware: Vec<String>,
    /// Pin binding files of every firmware bound to this board.
    pub pins_rs: Vec<PathBuf>,
    pub port: Option<u16>,
    /// The KiCad board file named by the manifest, if any.
    pub kicad: Option<PathBuf>,
    /// Root of the board's generated artifacts.
    pub out_dir: PathBuf,
}

/// Resolves the circuit entry file from CLI argument, `fusible.yml`, or convention.
pub fn resolve_entry(explicit: Option<PathBuf>) -> Result<PathBuf, TargetError> {
    resolve_board(explicit).map(|board| board.entry)
}

/// Resolves exactly one board; fails when the manifest has several and none was named.
pub fn resolve_board(explicit: Option<PathBuf>) -> Result<BoardTarget, TargetError> {
    let mut boards = resolve_boards(explicit)?;
    if boards.len() > 1 {
        let names = boards.into_iter().filter_map(|b| b.name).collect();
        return Err(TargetError::Ambiguous { names });
    }
    boards.pop().ok_or(TargetError::Missing)
}

/// Resolves the boards a command runs on.
///
/// `explicit` is a board name or a circuit file. Without it, every manifest board is
/// returned, or the conventional entry file when the manifest declares no board.
pub fn resolve_boards(explicit: Option<PathBuf>) -> Result<Vec<BoardTarget>, TargetError> {
    let cfg = FusibleConfig::nearest().map(|(_, c)| c).unwrap_or_default();
    if let Some(arg) = explicit {
        return Ok(vec![board_for(&cfg, arg)]);
    }
    if cfg.boards.is_empty() {
        let entry = entry_from_conventions().ok_or(TargetError::Missing)?;
        return Ok(vec![bare_board(&cfg, entry)]);
    }
    cfg.boards
        .iter()
        .map(|(name, board)| named_board(&cfg, name, board, None))
        .collect()
}

/// Resolves the board context of a circuit file, such as the one the dev server watches.
pub fn board_for_entry(entry: &Path) -> BoardTarget {
    let cfg = FusibleConfig::nearest().map(|(_, c)| c).unwrap_or_default();
    board_for(&cfg, entry.to_path_buf())
}

fn board_for(cfg: &FusibleConfig, arg: PathBuf) -> BoardTarget {
    let by_name = arg.to_str().and_then(|n| cfg.boards.get_key_value(n));
    if let Some((name, board)) = by_name {
        if let Ok(target) = named_board(cfg, name, board, None) {
            return target;
        }
    }
    let absolute = std::path::absolute(&arg).ok();
    let owner = cfg.boards.iter().find(|(_, b)| {
        let entry = b.entry.as_ref().or(b.circuit_json.as_ref());
        entry.is_some() && entry.and_then(|e| std::path::absolute(e).ok()) == absolute
    });
    match owner.or_else(|| cfg.boards.iter().next().filter(|_| cfg.boards.len() == 1)) {
        Some((name, board)) => named_board(cfg, name, board, Some(arg.clone()))
            .unwrap_or_else(|_| bare_board(cfg, arg)),
        None => bare_board(cfg, arg),
    }
}

fn named_board(
    cfg: &FusibleConfig,
    name: &str,
    board: &BoardConfig,
    entry: Option<PathBuf>,
) -> Result<BoardTarget, TargetError> {
    let entry = entry
        .or_else(|| board.entry.clone())
        .or_else(|| board.circuit_json.clone())
        .or_else(entry_from_conventions)
        .ok_or(TargetError::Missing)?;
    let out_dir = board
        .out_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("dist"));
    Ok(BoardTarget {
        name: Some(name.to_string()),
        entry,
        title: cfg.board_title(name),
        mcu: board.mcu.clone(),
        firmware: firmware_for(cfg, Some(name)),
        pins_rs: pins_for(cfg, Some(name)),
        port: board.port,
        kicad: board.kicad.clone(),
        out_dir: match cfg.boards.len() {
            0 | 1 => out_dir,
            _ => out_dir.join(name),
        },
    })
}

fn bare_board(cfg: &FusibleConfig, entry: PathBuf) -> BoardTarget {
    let (firmware, pins_rs) = match cfg.boards.is_empty() {
        true => (firmware_for(cfg, None), pins_for(cfg, None)),
        false => (Vec::new(), Vec::new()),
    };
    BoardTarget {
        name: None,
        entry,
        title: cfg.name.clone().unwrap_or_else(|| "Hardware Board".into()),
        firmware,
        pins_rs,
        out_dir: PathBuf::from("dist"),
        ..Default::default()
    }
}

fn firmware_for(cfg: &FusibleConfig, board: Option<&str>) -> Vec<String> {
    cfg.firmware_for(board)
        .map(|(name, _)| name.clone())
        .collect()
}

fn pins_for(cfg: &FusibleConfig, board: Option<&str>) -> Vec<PathBuf> {
    cfg.firmware_for(board)
        .filter_map(|(_, fw)| fw.pins_rs.clone())
        .collect()
}
