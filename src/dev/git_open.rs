//! Finds the git work tree of the project, or says why there is none.

use std::path::{Path, PathBuf};

use super::git_run::{Git, Unrunnable};

const SCOPE: [&str; 5] = [
    ".",
    ":(exclude,glob)**/node_modules/**",
    ":(exclude,glob)**/dist/**",
    ":(exclude,glob)**/.fusible/**",
    ":(exclude,glob)**/*.lck",
];

/// The state of git for the project folder.
#[derive(Debug)]
pub(super) enum Opened {
    NotRepo,
    Missing,
    Ready(Git),
}

/// Appends the pathspec that leaves build output and lock files out of what is saved.
pub(super) fn scoped<'a>(head: &[&'a str]) -> Vec<&'a str> {
    head.iter()
        .copied()
        .chain(["--"])
        .chain(SCOPE.iter().copied())
        .collect()
}

/// The folder git keeps while a rebase is stopped, if one is.
pub(super) fn rebase_dir(git: &Git) -> Option<PathBuf> {
    ["rebase-merge", "rebase-apply"].iter().find_map(|name| {
        let path = git.text(&["rev-parse", "--git-path", name])?;
        Some(git.dir().join(path)).filter(|dir| dir.is_dir())
    })
}

fn has_git_marker(dir: &Path) -> bool {
    dir.ancestors().any(|d| d.join(".git").exists())
}

/// Opens the work tree that contains `dir`, running commands from its top folder.
pub(super) fn open(program: &str, dir: &Path) -> Opened {
    let git = Git::new(program, dir);
    match git.run(&["rev-parse", "--show-toplevel"]) {
        Err(Unrunnable::Missing) if has_git_marker(dir) => Opened::Missing,
        Ok(out) if out.ok() => Opened::Ready(git.at(PathBuf::from(out.stdout.trim()))),
        _ => Opened::NotRepo,
    }
}
