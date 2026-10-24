//! Lookups across the named sections of a loaded manifest.

use std::path::{Path, PathBuf};

use super::{BodyConfig, FirmwareConfig, FusibleConfig};

impl FusibleConfig {
    /// Loads the manifest nearest to the working directory, ignoring lookup errors.
    ///
    /// `main` reports manifest errors before any command runs, so callers can treat
    /// an unreadable manifest like a missing one.
    pub fn nearest() -> Option<(PathBuf, Self)> {
        Self::find_and_load(Path::new(".")).ok().flatten()
    }

    /// Returns the firmware bound to `board`, or every firmware when `board` is `None`.
    ///
    /// A firmware without `board:` belongs to the only board of the project.
    pub fn firmware_for<'a>(
        &'a self,
        board: Option<&'a str>,
    ) -> impl Iterator<Item = (&'a String, &'a FirmwareConfig)> + 'a {
        self.firmware.iter().filter(move |(_, fw)| {
            board.is_none() || fw.board.is_none() || fw.board.as_deref() == board
        })
    }

    /// Returns the board whose frame `body` is modeled in.
    ///
    /// A generated body without `board:` belongs to the only board of the project. An
    /// authored body without `board:` belongs to no board.
    pub fn body_board<'a>(&'a self, body: &'a BodyConfig) -> Option<&'a str> {
        let only = || {
            let mut names = self.boards.keys();
            names.next().filter(|_| names.next().is_none())
        };
        match (&body.board, &body.generate) {
            (Some(board), _) => Some(board),
            (None, Some(_)) => only().map(String::as_str),
            (None, None) => None,
        }
    }

    /// Display title for `board`: the project name for a single-board project, else the board name.
    pub fn board_title(&self, board: &str) -> String {
        match &self.name {
            Some(name) if self.boards.len() <= 1 => name.clone(),
            _ => board.to_string(),
        }
    }
}
