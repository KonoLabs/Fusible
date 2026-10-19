//! Diagnostic de parsing : miette affiche le fichier source et un label sur
//! la position exacte de l'erreur Serde, comme un `error[E0308]` de rustc :
//!
//! ```text
//! error: Circuit JSON invalide : expected `,` or `}` at line 4 column 1
//!    ╭─[circuit.json:4:1]
//!  3 │   { "type": "source_net"
//!  4 │ ]
//!    · ┬
//!    · ╰── la désérialisation a échoué ici
//! ```
//!
//! C'est tout l'objectif « beaux diagnostics » du cahier des charges : le
//! message dit *où* dans le fichier, pas seulement *quoi*.

use std::path::Path;

use miette::{NamedSource, SourceSpan};
use thiserror::Error;

/// Échec de désérialisation du Circuit JSON, contextualisé par le fichier.
#[derive(Debug, Error, miette::Diagnostic)]
#[error("Circuit JSON invalide : {message}")]
pub struct JsonDiagnostic {
    message: String,
    #[source_code]
    src: NamedSource<String>,
    #[label("la désérialisation a échoué ici")]
    span: SourceSpan,
}

impl JsonDiagnostic {
    pub(super) fn new(err: serde_json::Error, source: &str, path: &Path) -> Self {
        // serde_json rapporte une position 1-based (ligne, colonne) ; miette
        // veut un offset en octets dans la source. On reconstitue l'offset en
        // parcourant les lignes. Approximation assumée : serde_json compte
        // les colonnes en octets, donc juste pour de l'ASCII — assez pour un
        // label de diagnostic, et l'offset est borné à la taille du fichier.
        let offset = if err.line() == 0 {
            0
        } else {
            byte_offset_at(source, err.line(), err.column())
        };
        let len = if offset < source.len() { 1 } else { 0 };

        Self {
            message: err.to_string(),
            src: NamedSource::new(path.display().to_string(), source.to_string()),
            span: SourceSpan::new(offset.into(), len),
        }
    }
}

/// Offset en octets d'une position (ligne, colonne) 1-based.
fn byte_offset_at(source: &str, line: usize, column: usize) -> usize {
    let mut offset = 0usize;
    for (index, text) in source.lines().enumerate() {
        if index + 1 == line {
            let rel = text
                .char_indices()
                .nth(column.saturating_sub(1))
                .map(|(b, _)| b)
                .unwrap_or(text.len());
            return (offset + rel).min(source.len());
        }
        // + le '\n' séparateur (approximation : CRLF traité comme LF)
        offset += text.len() + 1;
    }
    source.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn erreur_de_syntaxe_produit_un_diagnostic_avec_la_source() {
        let err = parse("{\"type\": ", Path::new("circuit.json"))
            .expect_err("le JSON tronqué doit échouer");
        assert!(!err.message.is_empty());
        // Le diagnostic embarque bien le fichier nommé (affiché par miette).
        assert!(err.src.inner().contains("type"));
    }

    #[test]
    fn offset_pointe_en_debut_de_ligne_fautive() {
        let source = "{\"a\": 1,\n\"b\": }";
        let offset = byte_offset_at(source, 2, 1);
        assert_eq!(&source[offset..], "\"b\": }");
    }
}
