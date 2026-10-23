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
mod context;
mod diagnostics;
mod rules;
#[cfg(test)]
mod tests;

pub use context::LintContext;
pub use diagnostics::{Diagnostic, FuseBlown, Severity};
pub use rules::{
    BypassCapRule, DanglingReferenceRule, GhostNetRule, MalformedElementRule, PinMuxRule,
    SchematicScaleRule, ShortedNetsRule, TraceClearanceRule, UnconnectedNetRule,
};

// ---------------------------------------------------------------------------
// Rule trait + Linter
// ---------------------------------------------------------------------------

/// One verification pass.
pub trait Rule {
    /// Short rule name, reused as the diagnostic's error code.
    fn name(&self) -> &'static str;

    /// Whether this rule blows the fuse or only warns. Errors by default.
    fn severity(&self) -> Severity {
        Severity::Error
    }

    /// Runs the rule against the lint context. An empty vector = rule passed.
    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic>;
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
    /// diagnostics, automatically constructing the lowered netlist. Test-only:
    /// commands build their own context with [`Self::run_with_context`].
    #[cfg(test)]
    pub fn run(&self, circuit: &crate::parser::Circuit) -> Vec<Diagnostic> {
        let netlist = crate::netlist::Netlist::from_circuit(circuit);
        let ctx = LintContext::new(circuit, &netlist);
        self.run_with_context(&ctx)
    }

    /// Runs every pass against a pre-built lint context.
    pub fn run_with_context(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        self.rules
            .iter()
            .flat_map(|rule| {
                let severity = rule.severity();
                rule.check(ctx)
                    .into_iter()
                    .map(move |d| Diagnostic { severity, ..d })
            })
            .collect()
    }
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
            .with_rule(UnconnectedNetRule)
            .with_rule(MalformedElementRule)
            .with_rule(GhostNetRule)
            .with_rule(DanglingReferenceRule)
            .with_rule(ShortedNetsRule)
            .with_rule(PinMuxRule)
            .with_rule(BypassCapRule)
            .with_rule(TraceClearanceRule)
            .with_rule(SchematicScaleRule)
    }
}
