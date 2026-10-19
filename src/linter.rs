//! Step 3 — rules engine (the linter core).
//!
//! The linter is a list of independent passes: each [`Rule`] inspects the
//! whole AST and returns its violations. Adding an electrical rule
//! (decoupling cap distance, pull-ups, voltage compatibility) = one new type
//! + one line in `Linter::default()`; nothing else in the pipeline moves.
//!
//! **Fail-closed contract**: a rule may only skip an element when another
//! registered rule is responsible for flagging it — see how
//! `UnconnectedNetRule` leans on `MalformedElementRule`. Silence is never
//! the default of this engine.
//!
//! Diagnostics and their miette rendering live in the `diagnostics`
//! submodule: only the engine stays here.
mod diagnostics;
mod rules;
#[cfg(test)]
mod tests;

pub use diagnostics::{Diagnostic, FuseBlown};
pub use rules::{GhostNetRule, MalformedElementRule, UnconnectedNetRule};

use crate::parser::Circuit;

// ---------------------------------------------------------------------------
// Rule trait + Linter
// ---------------------------------------------------------------------------

/// One verification pass.
pub trait Rule {
    /// Short rule name, reused as the diagnostic's error code.
    fn name(&self) -> &'static str;

    /// Runs the rule against the AST. An empty vector = rule passed.
    fn check(&self, circuit: &Circuit) -> Vec<Diagnostic>;
}

/// The fuse: runs every pass and aggregates their violations.
pub struct Linter {
    rules: Vec<Box<dyn Rule>>,
}

impl Linter {
    /// An empty linter — rules are added explicitly with [`Self::with_rule`].
    /// Tests use this to exercise one rule in isolation.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Appends a rule. Registration order is display order: `run` is
    /// deterministic, so diagnostics never shuffle between runs.
    pub fn with_rule(mut self, rule: impl Rule + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    /// Runs every pass in registration order and concatenates their
    /// diagnostics.
    pub fn run(&self, circuit: &Circuit) -> Vec<Diagnostic> {
        self.rules
            .iter()
            .flat_map(|rule| rule.check(circuit))
            .collect()
    }
}

impl Default for Linter {
    fn default() -> Self {
        // Phase 1: three rules proving the architecture. The rest of the
        // electrical playbook (decoupling, pull-ups, voltage domains) joins
        // here in phase 2.
        Self::new()
            .with_rule(UnconnectedNetRule)
            .with_rule(MalformedElementRule)
            .with_rule(GhostNetRule)
    }
}
