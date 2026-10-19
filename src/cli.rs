//! Étape 1 — Définition de la ligne de commande.
//!
//! On isole `clap` dans son propre module pour que `main.rs` ne parle qu'un
//! type métier (`Cli`) et reste lisible : le flux Lire → Parser → Linter →
//! Diagnostics occupe déjà toute l'attention qu'il mérite.

use std::path::PathBuf;

use clap::Parser;

/// Fusible — linter électronique : le fusible saute, la génération est bloquée.
#[derive(Debug, Parser)]
#[command(
    name = "fusible",
    version,
    about = "Linter électronique : le fusible saute pour bloquer la génération firmware",
    long_about = None
)]
pub struct Cli {
    /// Chemin vers le Circuit JSON (généré par tscircuit)
    #[arg(short = 'i', long, value_name = "FICHIER")]
    pub input: PathBuf,

    /// Cible silicium, ex. `stm32g431` (utilisée en phase 2 pour le pin-muxing)
    #[arg(long, value_name = "MCU")]
    pub mcu: Option<String>,
}
