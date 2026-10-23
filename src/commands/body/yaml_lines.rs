//! Line scanning of `fusible.yml`, just enough to find one body entry without a YAML library.

use std::ops::Range;

pub(super) fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

pub(super) fn is_skippable(line: &str) -> bool {
    let text = line.trim();
    text.is_empty() || text.starts_with('#')
}

/// Splits `key: value` into the key and the trimmed value; `None` for any other shape.
pub(super) fn key_value(line: &str) -> Option<(&str, &str)> {
    let (key, rest) = line.trim_start().split_once(':')?;
    let plain = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    (plain && (rest.is_empty() || rest.starts_with(' '))).then_some((key, rest.trim()))
}

/// The lines under a top-level `bodies:` header, or `None` without one.
pub(super) fn bodies_section(lines: &[String]) -> Option<Range<usize>> {
    let header = lines.iter().position(|l| {
        l.strip_prefix("bodies:")
            .is_some_and(|rest| is_skippable(rest) || rest.trim_start().starts_with('#'))
    })?;
    let end = lines[header + 1..]
        .iter()
        .position(|l| !is_skippable(l) && indent_of(l) == 0)
        .map_or(lines.len(), |i| header + 1 + i);
    Some(header + 1..end)
}

/// Finds the entry `name:` among the direct children of a section; returns its line and indent.
pub(super) fn entry(
    lines: &[String],
    section: &Range<usize>,
    name: &str,
) -> Option<(usize, usize)> {
    let child = lines[section.clone()]
        .iter()
        .find(|l| !is_skippable(l))
        .map(|l| indent_of(l))?;
    let is_entry = |i: &usize| {
        let line = &lines[*i];
        !is_skippable(line)
            && indent_of(line) == child
            && key_value(line).is_some_and(|(key, _)| key == name)
    };
    section.clone().find(is_entry).map(|i| (i, child))
}

/// The end (exclusive) of the block that starts at `start`: deeper lines, minus trailing blanks.
pub(super) fn block_end(lines: &[String], start: usize, indent: usize, limit: usize) -> usize {
    let mut end = start + 1;
    for (i, line) in lines.iter().enumerate().take(limit).skip(start + 1) {
        if is_skippable(line) {
            continue;
        }
        if indent_of(line) <= indent {
            break;
        }
        end = i + 1;
    }
    end
}
