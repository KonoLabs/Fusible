//! Tests du moteur de règles : construction d'ASTs à la main + assertions
//! sur les violations rendues.
//!
//! Fichier frère de `linter.rs` (déclaration `#[cfg(test)] mod tests`) :
//! isoler les tests garde le module principal sous la limite de fonctions
//! par fichier de filet.

use super::*;
use crate::parser::{CircuitElement, Net, Port};

fn net(id: &str, name: &str) -> CircuitElement {
    CircuitElement::Net(Net {
        net_id: Some(id.to_string()),
        name: Some(name.to_string()),
    })
}

fn port(id: &str, net_id: &str) -> CircuitElement {
    CircuitElement::Port(Port {
        port_id: Some(id.to_string()),
        name: None,
        net_id: Some(net_id.to_string()),
    })
}

#[test]
fn un_net_sans_connexion_saut() {
    let circuit = Circuit {
        elements: vec![net("n1", "GND"), port("U1.1", "n1")],
    };

    let violations = Linter::default().run(&circuit);

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "unconnected_net");
    assert!(violations[0].message.contains("GND"));
    assert!(violations[0].message.contains("flottant"));
}

#[test]
fn un_net_correctement_route_passe() {
    let circuit = Circuit {
        elements: vec![net("n1", "+3V3"), port("U1.1", "n1"), port("C1.1", "n1")],
    };

    assert!(Linter::default().run(&circuit).is_empty());
}

#[test]
fn les_regles_s_accumulent() {
    // Deux nets flottants = deux diagnostics, pas seulement le premier.
    let circuit = Circuit {
        elements: vec![
            net("n1", "GND"),
            port("U1.1", "n1"),
            net("n2", "SDA"),
            port("U1.2", "n2"),
        ],
    };

    let violations = Linter::default().run(&circuit);
    assert_eq!(violations.len(), 2);
}

/// Règle de test : échoue toujours, pour vérifier le point d'extension.
struct AlwaysFails;

impl Rule for AlwaysFails {
    fn name(&self) -> &'static str {
        "always_fails"
    }

    fn check(&self, _circuit: &Circuit) -> Vec<Diagnostic> {
        vec![Diagnostic {
            rule: self.name().to_string(),
            message: "toujours en échec".to_string(),
            help: None,
        }]
    }
}

#[test]
fn with_rule_ajoute_une_passe_supplementaire() {
    let circuit = Circuit { elements: vec![] };

    let violations = Linter::default().with_rule(AlwaysFails).run(&circuit);

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "always_fails");
}
