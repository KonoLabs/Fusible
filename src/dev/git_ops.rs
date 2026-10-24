//! Save and Get updates: the two things the Sync panel does.

use super::git_error::{failed, Failure};
use super::git_open::scoped;
use super::git_resolve::conflicts;
use super::git_run::{Git, Out};
use super::git_share::{first_remote, has_upstream, pull, share, Shared};

const DEFAULT_MESSAGE: &str = "Saved from Fusible Studio";

/// What a finished step tells the person, and the files it changed in the work tree.
#[derive(Debug)]
pub(super) struct Done {
    pub message: String,
    pub changed: Vec<String>,
}

/// A finished step, or a failure to show.
pub(super) type Outcome = Result<Done, Failure>;

/// Runs a git command and turns a non-zero exit into a [`Failure`].
pub(super) fn step(git: &Git, args: &[&str]) -> Result<Out, Failure> {
    let out = git.run(args)?;
    if out.ok() {
        Ok(out)
    } else {
        Err(failed(&out))
    }
}

fn head(git: &Git) -> Option<String> {
    git.text(&["rev-parse", "--verify", "-q", "HEAD"])
}

fn changed_since(git: &Git, before: Option<String>) -> Vec<String> {
    let Some(before) = before else {
        return Vec::new();
    };
    let text = git.text(&["diff", "--name-only", &before, "HEAD"]);
    text.unwrap_or_default().lines().map(String::from).collect()
}

fn commit(git: &Git, message: &str) -> Result<bool, Failure> {
    step(git, &scoped(&["add", "-A"]))?;
    let staged = git.run(&["diff", "--cached", "--quiet"])?;
    match staged.code {
        Some(0) => return Ok(false),
        Some(1) => {}
        _ => return Err(failed(&staged)),
    }
    let known = git
        .text(&["config", "user.email"])
        .is_some_and(|e| !e.is_empty());
    let who = [
        "-c",
        "user.name=Fusible Studio",
        "-c",
        "user.email=studio@fusible.invalid",
    ];
    let mut args: Vec<&str> = if known { Vec::new() } else { who.to_vec() };
    args.extend(["commit", "-m", message]);
    step(git, &args)?;
    Ok(true)
}

fn blocked(git: &Git) -> Result<(), Failure> {
    if conflicts(git).is_empty() {
        return Ok(());
    }
    Err(Failure::new(
        409,
        "Some files need your decision first. Choose which version to keep.",
    ))
}

/// Saves every change, then shares: gets the others' work, puts yours on top, and sends it.
pub(super) fn save(git: &Git, message: &str) -> Outcome {
    blocked(git)?;
    let message = Some(message.trim()).filter(|m| !m.is_empty());
    let committed = commit(git, message.unwrap_or(DEFAULT_MESSAGE))?;
    let before = head(git);
    let shared = share(git)?;
    let message = match shared {
        Shared::Local => "Saved on this computer. Add a remote to share.",
        Shared::Waiting => "Saved, but some files need your decision.",
        Shared::Pushed if committed => "Saved and shared.",
        Shared::Pushed => "Everything was already saved and shared.",
    };
    Ok(Done {
        message: message.into(),
        changed: changed_since(git, before),
    })
}

/// Gets the others' work and puts yours on top of it.
pub(super) fn update(git: &Git) -> Outcome {
    blocked(git)?;
    if !has_upstream(git) {
        let message = match first_remote(git) {
            Some(_) => "Nothing is shared yet. Press Save to share your work first.",
            None => "This project is only on this computer. Add a remote to get updates.",
        };
        return Ok(Done {
            message: message.into(),
            changed: Vec::new(),
        });
    }
    let before = head(git);
    let clean = pull(git)?;
    let changed = changed_since(git, before);
    let message = match (clean, changed.len()) {
        (false, _) => "Updates arrived, but some files need your decision.".into(),
        (true, 0) => "You already have the latest work.".into(),
        (true, n) => format!("Got updates to {n} file{}.", if n == 1 { "" } else { "s" }),
    };
    Ok(Done { message, changed })
}
