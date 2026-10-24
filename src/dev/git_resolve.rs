//! Conflicts: which files are stuck, and keeping one side of them.

use super::git_error::Failure;
use super::git_open::rebase_dir;
use super::git_ops::{step, Done, Outcome};
use super::git_run::Git;

/// Which version of a conflicting file to keep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Keep {
    Mine,
    Theirs,
}

impl Keep {
    /// Reads the `keep` field of a resolve request.
    pub(super) fn parse(value: &str) -> Option<Self> {
        match value {
            "mine" => Some(Self::Mine),
            "theirs" => Some(Self::Theirs),
            _ => None,
        }
    }
}

/// The files that are in conflict right now.
pub(super) fn conflicts(git: &Git) -> Vec<String> {
    let text = git.text(&["diff", "--name-only", "--diff-filter=U"]);
    text.unwrap_or_default().lines().map(String::from).collect()
}

fn flag(git: &Git, keep: Keep) -> &'static str {
    let merging = git
        .run(&["rev-parse", "-q", "--verify", "MERGE_HEAD"])
        .is_ok_and(|out| out.ok());
    match (keep, merging) {
        (Keep::Mine, true) | (Keep::Theirs, false) => "--ours",
        _ => "--theirs",
    }
}

fn take(git: &Git, path: &str, keep: Keep) -> Result<(), Failure> {
    let side = flag(git, keep);
    let checkout = git.run(&["checkout", side, "--", path])?;
    if checkout.ok() {
        step(git, &["add", "--", path])?;
    } else {
        step(git, &["rm", "--quiet", "--", path])?;
    }
    Ok(())
}

fn finish_rebase(git: &Git) -> Result<bool, Failure> {
    let out = git.run(&["rebase", "--continue"])?;
    if out.ok() {
        return Ok(true);
    }
    if !conflicts(git).is_empty() {
        return Ok(false);
    }
    step(git, &["rebase", "--skip"])?;
    Ok(true)
}

/// Keeps one side of a conflicting file and carries on with the update when none are left.
pub(super) fn resolve(git: &Git, path: &str, keep: Keep) -> Outcome {
    if !conflicts(git).iter().any(|c| c == path) {
        return Err(Failure::new(
            400,
            "That file is not waiting for a decision.",
        ));
    }
    take(git, path, keep)?;
    let done = |message: &str| Done {
        message: message.into(),
        changed: vec![path.into()],
    };
    if !conflicts(git).is_empty() {
        return Ok(done("Kept. More files need your decision."));
    }
    if rebase_dir(git).is_some() && !finish_rebase(git)? {
        return Ok(done("Kept. Another file needs your decision."));
    }
    Ok(done("All decided. Press Save to share your work."))
}
