//! The sync state served at `GET /api/git`.

use std::collections::BTreeMap;

use serde::Serialize;

use super::git_error::{human, unrunnable};
use super::git_log::{conflict_authors, last_saves, Save};
use super::git_open::{rebase_dir, scoped};
use super::git_porcelain::{parse, Change};
use super::git_run::Git;

/// Everything the Sync panel shows.
#[derive(Debug, Serialize, Default)]
pub(super) struct Status {
    pub repo: bool,
    pub available: bool,
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub has_remote: bool,
    pub ahead: u32,
    pub behind: u32,
    pub changes: Vec<Change>,
    pub last_saves: Vec<Save>,
    pub conflicts: Vec<String>,
    pub conflict_authors: BTreeMap<String, String>,
    pub fetch_error: Option<String>,
}

impl Status {
    /// The status served when git is not installed but the folder is a repository.
    pub(super) fn without_git() -> Self {
        Self {
            repo: true,
            ..Self::default()
        }
    }
}

fn rebasing_branch(git: &Git) -> Option<String> {
    let head = std::fs::read_to_string(rebase_dir(git)?.join("head-name")).ok()?;
    Some(head.trim().trim_start_matches("refs/heads/").to_string())
}

fn fetch(git: &Git) -> Option<String> {
    match git.run(&["fetch", "--quiet"]) {
        Ok(out) if out.ok() => None,
        Ok(out) => Some(human(&out.stderr)),
        Err(why) => Some(unrunnable(&why)),
    }
}

fn porcelain(git: &Git) -> String {
    let head = ["status", "--porcelain=v2", "--branch", "-z"];
    let args = scoped(&[head.as_slice(), &["--untracked-files=all"]].concat());
    git.run(&args).map(|o| o.stdout).unwrap_or_default()
}

/// Reads the sync state; `check` contacts the remote first to learn about updates.
pub(super) fn status(git: &Git, check: bool) -> Status {
    let remotes = git.text(&["remote"]).unwrap_or_default();
    let fetch_error = (check && !remotes.is_empty()).then(|| fetch(git)).flatten();
    let parsed = parse(&porcelain(git));
    let conflicts: Vec<String> = parsed
        .changes
        .iter()
        .filter(|c| c.kind == "conflict")
        .map(|c| c.path.clone())
        .collect();
    Status {
        repo: true,
        available: true,
        branch: parsed.branch.or_else(|| rebasing_branch(git)),
        upstream: parsed.upstream,
        has_remote: !remotes.is_empty(),
        ahead: parsed.ahead,
        behind: parsed.behind,
        conflict_authors: conflict_authors(git, &conflicts),
        last_saves: last_saves(git),
        changes: parsed.changes,
        conflicts,
        fetch_error,
    }
}
