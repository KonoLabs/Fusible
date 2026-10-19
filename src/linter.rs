//! Étape 3 — Moteur de règles (rules engine) du linter.
//!
//! Le linter est une liste de passes indépendantes : chaque `Rule` regarde
//! l'AST complet et renvoie ses violations. Ajouter une règle électrique
//! (découplage, pull-ups, tensions) = ajouter un type + l'enregistrer dans
//! `Linter::default()`, sans toucher au reste du pipeline.
//!
//! Les types de diagnostic et leur rendu miette sont isolés dans le
//! sous-module `diagnostics` : ici ne vit que le moteur.
mod diagnostics;

pub use diagnostics::{Diagnostic, FuseBlown};

use crate::parser::Circuit;

// ---------------------------------------------------------------------------
// Trait Rule + Linter
// ---------------------------------------------------------------------------

/// Une passe de vérification.
pub trait Rule {
    /// Nom court de la règle, réutilisé comme `code` du diagnostic.
    fn name(&self) -> &'static str;

    /// Exécute la règle sur l'AST. Un vecteur vide = règle passée.
    fn check(&self, circuit: &Circuit) -> Vec<Diagnostic>;
}

/// Le fusible : exécute toutes les passes et agrège leurs violations.
pub struct Linter {
    rules: Vec<Box<dyn Rule>>,
}

impl Default for Linter {
    fn default() -> Self {
        Self {
            // Phase 1 : une seule règle, pour valider l'architecture.
            // Les suivantes (découplage, pull-ups, tensions) viendront en
            // phase 2 au même endroit.
            rules: vec![Box::new(UnconnectedNetRule)],
        }
    }
}

impl Linter {
    /// Ajoute une règle (utile pour les tests et les phases suivantes).
    // Consommé en phase 2 (mcu::, codegen::) : inutilisé hors tests aujourd'hui.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn with_rule(mut self, rule: impl Rule + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    /// Exécute les passes dans l'ordre d'enregistrement et fusionne leurs
    /// diagnostics : l'ordre est stable, donc les erreurs s'affichent dans
    /// un ordre déterministe d'une exécution à l'autre.
    pub fn run(&self, circuit: &Circuit) -> Vec<Diagnostic> {
        self.rules
            .iter()
            .flat_map(|rule| rule.check(circuit))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Règle Phase 1
// ---------------------------------------------------------------------------

/// Aucun net ne doit être « flottant ».
///
/// Un net relié à moins de deux pads/ports est un oubli de routage : sur le
/// PCB réel, ce signal n'ira nulle part. Le fusible saute avant que le bug
/// n'atteigne le firmware.
pub struct UnconnectedNetRule;

/// Nombre minimum de connexions pour qu'un net soit considéré comme routé.
const MIN_CONNECTIONS: usize = 2;

impl Rule for UnconnectedNetRule {
    fn name(&self) -> &'static str {
        "unconnected_net"
    }

    fn check(&self, circuit: &Circuit) -> Vec<Diagnostic> {
        circuit
            .nets()
            .filter_map(|net| {
                // Sans identifiant on ne peut pas relier le net à ses ports :
                // modèle partiel, on n'invente pas de violation.
                let net_id = net.net_id.as_deref()?;

                let connections = circuit
                    .ports()
                    .filter(|port| port.net_id.as_deref() == Some(net_id))
                    .count()
                    + circuit
                        .pads()
                        .filter(|pad| pad.net_id.as_deref() == Some(net_id))
                        .count();

                if connections >= MIN_CONNECTIONS {
                    return None;
                }

                let label = net.name.as_deref().unwrap_or(net_id);
                Some(Diagnostic {
                    rule: self.name().to_string(),
                    message: format!(
                        "le net « {label} » n'a que {connections} connexion(s) \
                         (minimum {MIN_CONNECTIONS}) : il est flottant"
                    ),
                    help: Some(
                        "chaque net doit relier au moins deux pads/ports — \
                         complète le routage dans tscircuit"
                            .to_string(),
                    ),
                })
            })
            .collect()
    }
}

// Les tests vivent dans `src/linter/tests.rs` : en comptant les helpers,
// ce module dépasserait la limite de fonctions par fichier de filet.
#[cfg(test)]
mod tests;
