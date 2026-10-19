//! Étape 2 — Parser « Circuit JSON » (modèle partiel).
//!
//! # Choix d'architecture : Serde et le JSON hétérogène
//!
//! Le Circuit JSON de tscircuit est un **tableau d'objets hétérogènes** :
//! chaque élément porte un champ `"type"` discriminant (`source_component`,
//! `source_net`, `source_port`, `source_pad`, `source_trace`, ...).
//!
//! ```json
//! [
//!   { "type": "source_component", ... },
//!   { "type": "source_net", ... }
//! ]
//! ```
//!
//! C'est canoniquement le format *internally tagged enum* de Serde :
//!
//! * `#[serde(tag = "type")]` désigne le champ discriminant : Serde lit
//!   d'abord `type`, puis délègue la désérialisation du **reste** de l'objet
//!   à la variante correspondante. Aucun « lis `type`, puis `match`, puis
//!   `from_value` » à la main — Serde le fait en une passe.
//! * Les variantes sont des **newtype variants** (`Component(Component)`) :
//!   chaque variante embarque sa structure fortement typée. Serde accepte un
//!   struct derrière une variante à tag interne (les champs sont lus comme
//!   s'ils étaient aplatis dans l'objet d'origine).
//! * `#[serde(rename = "...")]` mappe l'identifiant JSON littéral vers un nom
//!   Rust sémantique (`Component`, `Net`, `Pad`, ...). Le tag du JSON bouge
//!   rarement ; le nom Rust, lui, doit rester lisible dans le reste du code.
//! * `#[serde(other)]` sur la variante `Unknown` : les types que la Phase 1
//!   ne modélise pas (`silkscreen_text`, `footprint`, `schematic_box`, ...)
//!   sont **ignorés au lieu de faire échouer tout le parsing**. C'est le choix
//!   central de robustesse : le linter doit se plaindre de règles
//!   électriques, jamais du fait que notre modèle est partiel. (Contrainte
//!   Serde : `other` n'existe que pour les enums à tag interne/adjacent, sur
//!   une variante unitaire — c'est exactement notre cas.)
//! * Serde ignore silencieusement les champs inconnus **au sein** d'une
//!   variante : le modèle partiel peut grandir (géométrie, pin-muxing de
//!   phase 2, ...) sans casser les fichiers existants.
//! * Les `alias = "..."` tolèrent les deux conventions de nommage de
//!   l'écosystème tscircuit (camelCase `sourceComponentId` vs snake_case
//!   `source_component_id`). Un alias ne coûte rien et évite un « fusible »
//!   qui saute pour une faute de frappe de nommage : un faux positif tue la
//!   confiance dans l'outil.
//!
//! Le diagnostic de parsing (label façon rustc sur la position Serde) vit
//! dans le sous-module `diagnostic` : ici ne reste que le modèle de données.
//!
// Le modèle est volontairement *partiel* : Pad/Trace et certains champs ne
// sont consommés qu'en phase 2 (pin-muxing, codegen). On ne les code pas
// pour rien, et on ne les supprime pas non plus.
#![allow(dead_code)]

mod diagnostic;

pub use diagnostic::JsonDiagnostic;

use std::path::Path;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Enum taggé : la racine du parsing
// ---------------------------------------------------------------------------

/// Un élément du tableau Circuit JSON.
///
/// Chaque variante correspond à une valeur possible du champ `"type"`.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum CircuitElement {
    /// Un composant posé sur le circuit (résistance, MCU, condensateur...).
    #[serde(rename = "source_component")]
    Component(Component),
    /// Une connexion logique (le « net » : GND, +3V3, I2C_SDA...).
    #[serde(rename = "source_net")]
    Net(Net),
    /// Une broche / un port d'un composant.
    #[serde(rename = "source_port")]
    Port(Port),
    /// Un pad de montage (surface d'accroche physique sur le PCB).
    #[serde(rename = "source_pad")]
    Pad(Pad),
    /// Une piste de cuivre reliant deux ports.
    #[serde(rename = "source_trace")]
    Trace(Trace),
    /// Type non modélisé en Phase 1 : ignoré plutôt que rejeté.
    #[serde(other)]
    Unknown,
}

// ---------------------------------------------------------------------------
// Structures fortement typées
// ---------------------------------------------------------------------------

/// Un composant.
#[derive(Debug, Clone, Deserialize)]
pub struct Component {
    /// Identifiant stable, ex. `U1`.
    #[serde(alias = "sourceComponentId", alias = "source_component_id")]
    pub component_id: Option<String>,
    /// Nature du composant : `chip`, `resistor`, `capacitor`, ...
    #[serde(alias = "ftype")]
    pub kind: Option<String>,
}

/// Un net : la connexion logique entre broches.
#[derive(Debug, Clone, Deserialize)]
pub struct Net {
    #[serde(alias = "sourceNetId", alias = "source_net_id")]
    pub net_id: Option<String>,
    /// Nom lisible affiché dans les diagnostics, ex. `GND`.
    #[serde(alias = "netName")]
    pub name: Option<String>,
}

/// Une broche / un port de composant.
#[derive(Debug, Clone, Deserialize)]
pub struct Port {
    #[serde(alias = "sourcePortId", alias = "source_port_id")]
    pub port_id: Option<String>,
    #[serde(alias = "portName")]
    pub name: Option<String>,
    /// Net logique rattaché à ce port — la donnée que la règle
    /// `UnconnectedNetRule` vient compter.
    #[serde(alias = "sourceNetId", alias = "source_net_id", alias = "netId")]
    pub net_id: Option<String>,
}

/// Un pad de montage (l'analogue physique d'un port).
#[derive(Debug, Clone, Deserialize)]
pub struct Pad {
    #[serde(alias = "sourcePadId", alias = "source_pad_id")]
    pub pad_id: Option<String>,
    #[serde(alias = "sourceNetId", alias = "source_net_id", alias = "netId")]
    pub net_id: Option<String>,
}

/// Une piste de cuivre.
#[derive(Debug, Clone, Deserialize)]
pub struct Trace {
    #[serde(alias = "sourceTraceId", alias = "source_trace_id")]
    pub trace_id: Option<String>,
    /// Ports reliés par cette piste.
    #[serde(default, alias = "connectedSourcePortIds")]
    pub connected_port_ids: Vec<String>,
}

// ---------------------------------------------------------------------------
// AST : vue d'ensemble indexée par type
// ---------------------------------------------------------------------------

/// L'AST du circuit : le tableau d'éléments brut plus des accès typés.
///
/// Plutôt que de faire `match` sur `elements` partout dans le code, on
/// centralise ici les « vues » par type. Le linter et (plus tard) le module
/// `mcu::` consomment ces itérateurs sans connaître la forme du JSON.
#[derive(Debug, Clone)]
pub struct Circuit {
    pub elements: Vec<CircuitElement>,
}

impl Circuit {
    pub fn components(&self) -> impl Iterator<Item = &Component> {
        self.elements.iter().filter_map(|e| match e {
            CircuitElement::Component(c) => Some(c),
            _ => None,
        })
    }

    pub fn nets(&self) -> impl Iterator<Item = &Net> {
        self.elements.iter().filter_map(|e| match e {
            CircuitElement::Net(n) => Some(n),
            _ => None,
        })
    }

    pub fn ports(&self) -> impl Iterator<Item = &Port> {
        self.elements.iter().filter_map(|e| match e {
            CircuitElement::Port(p) => Some(p),
            _ => None,
        })
    }

    pub fn pads(&self) -> impl Iterator<Item = &Pad> {
        self.elements.iter().filter_map(|e| match e {
            CircuitElement::Pad(p) => Some(p),
            _ => None,
        })
    }

    pub fn traces(&self) -> impl Iterator<Item = &Trace> {
        self.elements.iter().filter_map(|e| match e {
            CircuitElement::Trace(t) => Some(t),
            _ => None,
        })
    }
}

// ---------------------------------------------------------------------------
// Point d'entrée du parsing
// ---------------------------------------------------------------------------

/// Désérialise le tableau Circuit JSON.
///
/// `path` sert uniquement à nommer le fichier dans le diagnostic.
pub fn parse(source: &str, path: &Path) -> Result<Circuit, JsonDiagnostic> {
    let elements: Vec<CircuitElement> =
        serde_json::from_str(source).map_err(|err| JsonDiagnostic::new(err, source, path))?;
    Ok(Circuit { elements })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_un_tableau_heterogene() {
        let json = r#"[
            {"type": "source_component", "component_id": "U1", "kind": "chip"},
            {"type": "source_net", "net_id": "n1", "name": "GND"},
            {"type": "source_port", "port_id": "U1.1", "name": "VSS", "net_id": "n1"},
            {"type": "source_pad", "pad_id": "p1", "net_id": "n1"},
            {"type": "source_trace", "trace_id": "t1", "connected_port_ids": ["U1.1", "U1.2"]},
            {"type": "silkscreen_text", "text": "non modélisé en phase 1"}
        ]"#;

        let circuit = parse(json, Path::new("circuit.json")).expect("le tableau doit parser");

        assert_eq!(circuit.elements.len(), 6);
        assert_eq!(circuit.components().count(), 1);
        assert_eq!(circuit.nets().count(), 1);
        assert_eq!(circuit.ports().count(), 1);
        assert_eq!(circuit.pads().count(), 1);
        assert_eq!(circuit.traces().count(), 1);
        // Le type inconnu est ignoré, pas rejeté.
        assert!(matches!(circuit.elements[5], CircuitElement::Unknown));
    }

    #[test]
    fn accepte_le_camelcase_tscircuit() {
        let json =
            r#"[{"type": "source_component", "sourceComponentId": "R1", "ftype": "resistor"}]"#;
        let circuit = parse(json, Path::new("circuit.json")).expect("alias camelCase acceptés");
        let component = circuit.components().next().expect("un composant");
        assert_eq!(component.component_id.as_deref(), Some("R1"));
        assert_eq!(component.kind.as_deref(), Some("resistor"));
    }
}
