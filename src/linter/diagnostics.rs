//! Diagnostics du linter et leur rendu miette.
//!
//! Deux niveaux :
//! * [`Diagnostic`] — une violation individuelle (le « où et pourquoi ») ;
//! * [`FuseBlown`] — l'erreur agrégée rendue par `main` quand le linter a
//!   trouvé au moins une violation : titre + violations en *erreurs liées*,
//!   puis exit code ≠ 0 pour bloquer la génération.

use std::fmt;

use thiserror::Error;

// ---------------------------------------------------------------------------
// Une violation
// ---------------------------------------------------------------------------

/// Une violation détectée par une règle.
///
/// On implémente `miette::Diagnostic` **à la main** plutôt que par la macro
/// dérivée : le `code` (affiché `error[unconnected_net]`) et le `help`
/// dépendent de la règle qui a échoué — ce sont des données d'instance, et
/// le dérivateur de miette ne sait produire que des littéraux fixes au
/// niveau de la structure.
#[derive(Debug, Error)]
#[error("[{rule}] {message}")]
pub struct Diagnostic {
    /// Identifiant stable de la règle, ex. `unconnected_net`.
    pub rule: String,
    /// Message façon compilateur Rust : où ça pète et pourquoi.
    pub message: String,
    /// Aide contextuelle affichée en `help: ...` sous le diagnostic.
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

// ---------------------------------------------------------------------------
// Le fusible saute
// ---------------------------------------------------------------------------

/// Agrège toutes les violations en un seul rapport d'erreur.
///
/// Retourné par `main` quand le linter a trouvé quelque chose : miette
/// affiche ce titre, puis chaque `Diagnostic` en erreur liée, et le
/// processus sort avec un code ≠ 0 — ce qui bloquera la chaîne de
/// génération (`codegen::` n'est jamais appelé).
#[derive(Debug, Error)]
#[error("le fusible a sauté : {} violation(s) électrique(s)", .violations.len())]
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
            "corrige les violations ci-dessus : tant que le fusible saute, \
             pins.rs n'est pas généré",
        ))
    }
}
