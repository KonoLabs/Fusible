//! Finding a component's JSX tag and rewriting a literal number prop in it.

use std::ops::Range;

/// Returns the byte range of the single JSX opening tag with `name="<designator>"`.
///
/// `None` when no tag or several tags carry that name: the part cannot be located safely.
pub(super) fn find_tag(source: &str, designator: &str) -> Option<Range<usize>> {
    let needle = format!("name=\"{designator}\"");
    let mut hits = source.match_indices(&needle).map(|(at, _)| at);
    let at = hits.next()?;
    if hits.next().is_some() {
        return None;
    }
    let start = source[..at].rfind('<')?;
    let end = tag_end(source, start)?;
    Some(start..end)
}

/// Returns the offset just past the `>` that closes the tag opened at `start`.
fn tag_end(source: &str, start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    for (i, c) in source[start..].char_indices() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '{') => depth += 1,
            (None, '}') => depth = depth.checked_sub(1)?,
            (None, '>') if depth == 0 => return Some(start + i + 1),
            _ => {}
        }
    }
    None
}

/// Returns `tag` with the literal number of `prop` replaced by `value`.
///
/// Handles `prop={12.5}` and `prop="12.5mm"` (or any unit); `None` when the prop is absent or
/// is an expression such as `prop={x + 1}`.
pub(super) fn with_number(tag: &str, prop: &str, value: f64) -> Option<String> {
    let (open, body) = prop_body(tag, prop)?;
    let digits = tag[body.clone()].trim_start();
    let number_len = digits
        .find(|c: char| !(c.is_ascii_digit() || matches!(c, '-' | '+' | '.')))
        .unwrap_or(digits.len());
    digits[..number_len].parse::<f64>().ok()?;
    let unit = &digits[number_len..];
    if open == '{' && !unit.trim().is_empty() {
        return None;
    }
    let (head, tail) = (&tag[..body.start], &tag[body.end..]);
    Some(format!("{head}{}{unit}{tail}", format_mm(value)))
}

/// Returns the opening delimiter of `prop`'s value and the byte range between the delimiters.
fn prop_body(tag: &str, prop: &str) -> Option<(char, Range<usize>)> {
    let key = format!("{prop}=");
    let at = tag
        .match_indices(&key)
        .find(|(i, _)| tag[..*i].ends_with(char::is_whitespace))?
        .0
        + key.len();
    let (open, close) = match tag[at..].chars().next()? {
        '{' => ('{', '}'),
        '"' => ('"', '"'),
        _ => return None,
    };
    let start = at + 1;
    let end = start + tag[start..].find(close)?;
    Some((open, start..end))
}

/// Formats a millimetre value with at most three decimals and no trailing zeros.
fn format_mm(value: f64) -> String {
    let text = format!("{value:.3}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    match trimmed {
        "-0" | "" => "0".to_string(),
        other => other.to_string(),
    }
}
