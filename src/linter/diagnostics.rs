//! Linter diagnostics and their miette rendering.
//!
//! Two levels:
//! * [`Diagnostic`] — one individual violation (the *where* and *why*);
//! * [`FuseBlown`] — the aggregate error `main` renders when the linter
//!   found at least one violation: a title with the violations as *related
//!   errors*, then exit code 1 so codegen never runs.

use std::fmt;

use thiserror::Error;

/// A violation detected by a rule.
///
/// `miette::Diagnostic` is implemented **by hand** instead of derived: the
/// `code` (rendered as `error[unconnected_net]`) and the `help` depend on
/// which rule fired — they are instance data, and the derive macro only
/// produces fixed literals at the struct level. The `code` also replaces the
/// rule prefix the `Display` used to carry, so miette no longer prints the
/// rule name twice.
#[derive(Debug, Error)]
#[error("{message}")]
pub struct Diagnostic {
    /// Stable rule identifier, e.g. `unconnected_net`.
    pub rule: String,
    /// Compiler-style message: what broke and why.
    pub message: String,
    /// Contextual help rendered as `help: ...` under the diagnostic.
    pub help: Option<String>,
}

impl miette::Diagnostic for Diagnostic {
    fn code(&self) -> Option<Box<dyn fmt::Display + '_>> {
        Some(Box::new(self.rule.as_str()))
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn help(&self) -> Option<Box<dyn fmt::Display + '_>> {
        self.help
            .as_deref()
            .map(|help| Box::new(help) as Box<dyn fmt::Display>)
    }
}

/// Aggregates every violation into a single error report.
///
/// Returned by `main` when the linter found something: miette renders this
/// title, then each `Diagnostic` as a related error, and the process exits
/// with code 1 — which is what blocks the generation chain (`codegen::` is
/// never called).
#[derive(Debug, Error)]
#[error("the fuse blew: {} electrical violation(s)", .violations.len())]
pub struct FuseBlown {
    pub violations: Vec<Diagnostic>,
}

impl FuseBlown {
    pub fn new(violations: Vec<Diagnostic>) -> Self {
        Self { violations }
    }
}

impl miette::Diagnostic for FuseBlown {
    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
        Some(Box::new(
            self.violations
                .iter()
                .map(|diagnostic| diagnostic as &dyn miette::Diagnostic),
        ))
    }

    fn help(&self) -> Option<Box<dyn fmt::Display + '_>> {
        Some(Box::new(
            "fix the violations above: generation stays blocked while the \
             fuse is blown",
        ))
    }
}
