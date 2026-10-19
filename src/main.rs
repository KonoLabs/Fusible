//! Fusible — Phase 1 (MVP).
//!
//! Flux : Lire le fichier → Parser → Linter → afficher les diagnostics via
//! miette, ou « ✅ Fusible intact » si aucun diagnostic.
//!
//! Les étapes suivantes du plan global (`mcu::`, `codegen::`) sont
//! délibérément absentes : elles ne sont activées qu'après validation de
//! cette phase, et le fusible doit sauter *avant* toute génération de code.
mod cli;
mod linter;
mod parser;

use clap::Parser;
use thiserror::Error;

use crate::cli::Cli;
use crate::linter::{FuseBlown, Linter};

/// Fichier d'entrée illisible : on rend l'erreur lisible plutôt qu'un
/// `unwrap()` avec une backtrace brute.
#[derive(Debug, Error, miette::Diagnostic)]
#[error("impossible de lire le Circuit JSON « {path} »")]
#[diagnostic(help("vérifie que l'argument --input pointe vers un fichier existant"))]
struct ReadError {
    path: String,
    #[source]
    source: std::io::Error,
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    // 1. Lecture du fichier en une seule chaîne : elle sert de *source_code*
    //    aux diagnostics miette (le parser a besoin du texte pour placer les
    //    labels, comme un compilateur qui remonte la ligne fautive).
    let source = std::fs::read_to_string(&cli.input).map_err(|source| ReadError {
        path: cli.input.display().to_string(),
        source,
    })?;

    // 2. Parsing : un échec sort déjà avec un diagnostic façon rustc.
    let circuit = parser::parse(&source, &cli.input)?;

    // 3. Linter : toutes les passes, tous les diagnostics.
    let violations = Linter::default().run(&circuit);

    // 4. Verdict du fusible.
    if violations.is_empty() {
        println!("✅ Circuit validé (Fusible intact)");
        if let Some(mcu) = cli.mcu.as_deref() {
            println!("   cible silicium : {mcu} (pin-muxing : phase 2)");
        }
        return Ok(());
    }

    // Le fusible saute : l'erreur agrégée est rendue par miette (chaque
    // violation en erreur liée) et le processus sort avec un code ≠ 0,
    // ce qui bloquera la génération de pins.rs en phase 2.
    Err(FuseBlown::new(violations).into())
}
