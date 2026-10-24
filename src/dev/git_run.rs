//! Runs the real git binary: no shell, a deadline, and never a prompt.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use super::git_proc::{collect, drain, quiet, wait};

const TIMEOUT: Duration = Duration::from_secs(30);

/// What a finished git command printed and how it ended.
#[derive(Debug, Clone)]
pub(super) struct Out {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Out {
    pub(super) fn ok(&self) -> bool {
        self.code == Some(0)
    }
}

/// Why git could not run at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Unrunnable {
    Missing,
    TimedOut,
    Io(String),
}

/// A git binary and the directory its commands run in.
#[derive(Debug, Clone)]
pub(super) struct Git {
    program: OsString,
    dir: PathBuf,
    timeout: Duration,
}

impl Git {
    pub(super) fn new(program: impl Into<OsString>, dir: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            dir: dir.into(),
            timeout: TIMEOUT,
        }
    }

    #[cfg(all(test, unix))]
    pub(super) fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub(super) fn dir(&self) -> &Path {
        &self.dir
    }

    pub(super) fn at(&self, dir: PathBuf) -> Self {
        Self {
            dir,
            ..self.clone()
        }
    }

    /// Runs `git args` without a shell or a terminal prompt, and kills it after the deadline.
    pub(super) fn run(&self, args: &[&str]) -> Result<Out, Unrunnable> {
        let mut command = Command::new(&self.program);
        command
            .args(args)
            .current_dir(&self.dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        quiet(&mut command);
        let mut child = command.spawn().map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Unrunnable::Missing,
            _ => Unrunnable::Io(e.to_string()),
        })?;
        let stdout = drain(child.stdout.take());
        let stderr = drain(child.stderr.take());
        let status = wait(&mut child, self.timeout)?;
        Ok(Out {
            code: status.code(),
            stdout: collect(stdout),
            stderr: collect(stderr),
        })
    }

    /// Runs `git args` and returns the trimmed stdout when it succeeded.
    pub(super) fn text(&self, args: &[&str]) -> Option<String> {
        let out = self.run(args).ok().filter(Out::ok)?;
        Some(out.stdout.trim().to_string())
    }
}
