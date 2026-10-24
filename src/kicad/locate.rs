//! Finding the KiCad board file of a tscircuit board.

use std::path::{Path, PathBuf};

/// Returns the KiCad board file of a board.
///
/// The manifest's `kicad:` path wins. Otherwise the only `.kicad_pcb` beside the entry file
/// or in a `kicad/` folder next to it is used; with none or several there is no answer.
pub fn board_file(configured: Option<&Path>, entry: &Path) -> Option<PathBuf> {
    if let Some(path) = configured {
        return Some(path.to_path_buf());
    }
    let dir = match entry.is_dir() {
        true => entry,
        false => entry.parent()?,
    };
    let dir = match dir.as_os_str().is_empty() {
        true => Path::new("."),
        false => dir,
    };
    only_board(dir).or_else(|| only_board(&dir.join("kicad")))
}

/// The single `.kicad_pcb` file directly inside `dir`.
fn only_board(dir: &Path) -> Option<PathBuf> {
    let mut boards = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "kicad_pcb"));
    let first = boards.next()?;
    boards.next().is_none().then_some(first)
}
