//! Turns git failures into sentences a person can act on.

use super::git_run::{Out, Unrunnable};

const DETAILS: usize = 600;

/// A failed sync step: an HTTP code, a plain sentence, and the raw output for the curious.
#[derive(Debug)]
pub(super) struct Failure {
    pub code: u16,
    pub error: String,
    pub details: String,
}

impl Failure {
    pub(super) fn new(code: u16, error: &str) -> Self {
        Self {
            code,
            error: error.into(),
            details: String::new(),
        }
    }
}

/// A plain sentence for git's error output.
pub(super) fn human(stderr: &str) -> String {
    let text = stderr.to_ascii_lowercase();
    let has = |needles: &[&str]| needles.iter().any(|n| text.contains(n));
    let message = if has(&[
        "authentication failed",
        "terminal prompts disabled",
        "permission denied",
        "could not read username",
        "publickey",
    ]) {
        "The server did not accept your sign-in. Sign in once with Git or set up an SSH key, then try again."
    } else if has(&[
        "could not resolve host",
        "unable to access",
        "connection",
        "network",
    ]) {
        "Cannot reach the server. Check your internet connection and try again."
    } else if has(&["non-fast-forward", "[rejected]", "fetch first"]) {
        "Someone saved before you. Press Get updates, then Save again."
    } else if has(&["author identity unknown", "unable to auto-detect email"]) {
        "Git does not know who you are yet. Set your name and email in Git, then save again."
    } else if has(&["hook"]) {
        "A rule of this project refused the save. See the details."
    } else {
        "That did not work. See the details."
    };
    message.into()
}

/// A plain sentence for a git command that could not run.
pub(super) fn unrunnable(why: &Unrunnable) -> String {
    match why {
        Unrunnable::Missing => "Git is not installed.".into(),
        Unrunnable::TimedOut => "The server took too long to answer. Try again.".into(),
        Unrunnable::Io(reason) => format!("Git could not start: {reason}"),
    }
}

fn clip(text: &str) -> String {
    text.trim().chars().take(DETAILS).collect()
}

impl From<Unrunnable> for Failure {
    fn from(why: Unrunnable) -> Self {
        let code = if why == Unrunnable::Missing { 503 } else { 504 };
        Self::new(code, &unrunnable(&why))
    }
}

/// The failure for a git command that ran and exited with an error.
pub(super) fn failed(out: &Out) -> Failure {
    Failure {
        code: 500,
        error: human(&out.stderr),
        details: clip(&format!("{}\n{}", out.stderr, out.stdout)),
    }
}
