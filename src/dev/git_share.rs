//! Sharing saved work: get the others' work, put yours on top, send it.

use super::git_error::{failed, Failure};
use super::git_ops::step;
use super::git_resolve::conflicts;
use super::git_run::Git;

/// How far a share got.
pub(super) enum Shared {
    Local,
    Waiting,
    Pushed,
}

/// Whether the upstream branch has a shared copy to get updates from.
pub(super) fn has_upstream(git: &Git) -> bool {
    git.text(&["rev-parse", "--abbrev-ref", "@{upstream}"])
        .is_some()
}

/// Gets the others' work under the unsaved changes; `false` when files need a decision.
pub(super) fn pull(git: &Git) -> Result<bool, Failure> {
    let out = git.run(&["pull", "--rebase", "--autostash"])?;
    if !conflicts(git).is_empty() {
        return Ok(false);
    }
    if out.ok() {
        Ok(true)
    } else {
        Err(failed(&out))
    }
}

/// The remote work is shared with: `origin` when there is one, else the first.
pub(super) fn first_remote(git: &Git) -> Option<String> {
    let remotes = git.text(&["remote"])?;
    let first = remotes.lines().next()?;
    Some(
        remotes
            .lines()
            .find(|r| *r == "origin")
            .unwrap_or(first)
            .into(),
    )
}

/// Pulls and pushes, or only reports that the work stays on this computer.
pub(super) fn share(git: &Git) -> Result<Shared, Failure> {
    let Some(remote) = first_remote(git) else {
        return Ok(Shared::Local);
    };
    if has_upstream(git) {
        if !pull(git)? {
            return Ok(Shared::Waiting);
        }
        step(git, &["push"])?;
        return Ok(Shared::Pushed);
    }
    let branch = git
        .text(&["symbolic-ref", "--short", "HEAD"])
        .unwrap_or_default();
    step(git, &["push", "--set-upstream", &remote, &branch])?;
    Ok(Shared::Pushed)
}
