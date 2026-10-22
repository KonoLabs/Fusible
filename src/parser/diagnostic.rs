//! Parse diagnostic: miette prints the source file with a label on the
//! position Serde reported, the way rustc prints an `error[E0308]`:
//!
//! ```text
//! error: invalid Circuit JSON: expected `,` or `}` at line 4 column 1
//!    ╭─[circuit.json:4:1]
//!  3 │   { "type": "source_net"
//!  4 │ ]
//!    · ┬
//!    · ╰── deserialization failed here
//! ```
//!
//! That is the whole "pretty diagnostics" requirement of the spec: the
//! message says *where* in the file, not only *what*.

use std::path::Path;

use miette::{NamedSource, SourceSpan};
use thiserror::Error;

/// A Circuit JSON deserialization failure, anchored to its file.
#[derive(Debug, Error, miette::Diagnostic)]
#[error("invalid Circuit JSON: {message}")]
pub struct JsonDiagnostic {
    pub message: String,
    #[source_code]
    pub src: NamedSource<String>,
    #[label("deserialization failed here")]
    pub span: SourceSpan,
}

impl JsonDiagnostic {
    pub(super) fn new(err: serde_json::Error, source: &str, path: &Path) -> Self {
        Self::at(err.to_string(), &err, source, path)
    }

    /// A failure inside element `index` of an otherwise valid JSON array.
    ///
    /// Errors from an in-memory value carry no position, so the source is
    /// re-read as typed elements on this error path only: serde_json then
    /// reports where the same element fails in the file.
    pub(super) fn element(index: usize, err: serde_json::Error, source: &str, path: &Path) -> Self {
        let message = format!("element #{index}: {err}");
        match serde_json::from_str::<Vec<super::CircuitElement>>(source) {
            Err(located) => Self::at(message, &located, source, path),
            Ok(_) => Self::at(message, &err, source, path),
        }
    }

    /// Anchors `message` on the (line, column) serde_json reported, mapped to
    /// a byte offset clamped inside the file so a degenerate position renders
    /// a zero-length label instead of panicking.
    fn at(message: String, located: &serde_json::Error, source: &str, path: &Path) -> Self {
        let offset = byte_offset_at(source, located.line(), located.column());
        let len = if offset < source.len() { 1 } else { 0 };

        Self {
            message,
            src: NamedSource::new(path.display().to_string(), source.to_string()),
            span: SourceSpan::new(offset.into(), len),
        }
    }
}

/// Byte offset of a serde_json (line, column) position.
///
/// serde_json counts the column in **bytes** since the start of the line
/// (1-based for syntax errors; bytes-consumed for EOF errors — both land
/// sensibly once clamped). Two traps this handles:
///
/// * CRLF: `str::lines()` strips the `\r`, so a naive `len + 1` walk loses
///   one byte per preceding line and the label drifts left. We walk
///   `split_inclusive('\n')` segments instead, `\r` included.
/// * Multi-byte characters: a byte column can land *inside* a character; the
///   offset walks back to a `char` boundary so miette never gets a position
///   that splits UTF-8.
///
/// Returns `source.len()` when the line does not exist (label at EOF).
pub(super) fn byte_offset_at(source: &str, line: usize, column: usize) -> usize {
    let mut remaining = line.max(1);
    let mut offset = 0usize;

    for segment in source.split_inclusive('\n') {
        if remaining == 1 {
            let text = segment.strip_suffix('\n').unwrap_or(segment);
            let mut rel = column.saturating_sub(1).min(text.len());
            while rel > 0 && !text.is_char_boundary(rel) {
                rel -= 1;
            }
            return (offset + rel).min(source.len());
        }
        offset += segment.len();
        remaining -= 1;
    }

    source.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn rejected_input_carries_the_named_source() {
        // `{"type": ` at the root is rejected as "map, expected a
        // sequence" — the object-root case, the most common user error.
        let err = parse("{\"type\": ", Path::new("circuit.json"))
            .expect_err("an object root must not parse");
        assert!(!err.message.is_empty());
        // The diagnostic embeds the named file miette renders.
        assert!(err.src.inner().contains("type"));
    }

    #[test]
    fn span_reaches_the_failing_line_of_a_multiline_file() {
        let source = "[\n  {\"type\": \"source_net\"},\n  {,}\n]";
        let line3_start = source.find("\n  {,}").expect("line 3") + 1;
        let line3_end = line3_start + source[line3_start..].find('\n').expect("line 3 end");

        let err = parse(source, Path::new("circuit.json")).expect_err("bad element");
        let offset = err.span.offset();
        assert!(
            (line3_start..line3_end).contains(&offset),
            "span {offset} not on line 3 ({line3_start}..{line3_end})"
        );
    }

    #[test]
    fn empty_input_renders_a_zero_length_span_at_the_start() {
        let err = parse("", Path::new("circuit.json")).expect_err("empty input");
        assert!(err.message.contains("EOF"));
        assert_eq!(err.span.offset(), 0);
        assert_eq!(err.span.len(), 0);
    }
}
