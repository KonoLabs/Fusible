//! Rewrites one generated body entry of `fusible.yml` into an authored one.
//!
//! The edit is textual so comments and layout survive. It handles the two shapes people
//! write (a block mapping and a one-line `{ }` mapping) and gives up on anything else.

use super::yaml_lines::{block_end, bodies_section, entry, indent_of, is_skippable, key_value};

/// Settings that only matter while Fusible generates the body.
const GENERATION_KEYS: &[&str] = &[
    "format",
    "out_dir",
    "out-dir",
    "wall_thickness",
    "wall-thickness",
    "height",
    "cutouts",
    "ventilation",
    "mounting",
];

enum Action {
    Source,
    Drop,
    Keep,
}

fn classify(key: &str) -> Result<Action, String> {
    match key {
        "source" => Err("the body already has a `source:`".into()),
        "generate" => Ok(Action::Source),
        k if GENERATION_KEYS.contains(&k) => Ok(Action::Drop),
        _ => Ok(Action::Keep),
    }
}

fn edit_block(body: &[String], path: &str) -> Result<Vec<String>, String> {
    let child = body
        .iter()
        .find(|l| !is_skippable(l))
        .map(|l| indent_of(l))
        .ok_or("the body has no settings")?;
    let (mut out, mut replaced) = (Vec::new(), false);
    for line in body {
        if is_skippable(line) {
            out.push(line.clone());
            continue;
        }
        if indent_of(line) != child {
            return Err("a setting spans several lines".into());
        }
        let (key, _) = key_value(line).ok_or("a line is not a plain `key: value`")?;
        match classify(key)? {
            Action::Source => {
                out.push(format!("{}source: {path}", " ".repeat(child)));
                replaced = true;
            }
            Action::Keep => out.push(line.clone()),
            Action::Drop => {}
        }
    }
    replaced
        .then_some(out)
        .ok_or_else(|| "the body has no `generate:`".into())
}

/// Splits `{ inner }  # tail` into the inside of the braces and the trailing comment.
fn split_flow(rest: &str) -> Result<(&str, &str), String> {
    let close = rest
        .rfind('}')
        .ok_or("the `{ }` mapping spans several lines")?;
    let (inner, tail) = (&rest[1..close], rest[close + 1..].trim());
    if !(tail.is_empty() || tail.starts_with('#')) {
        return Err("text follows the `{ }` mapping".into());
    }
    if inner.contains(['{', '[', ']', '"', '\'', '#']) {
        return Err("the `{ }` mapping holds quotes or nested values".into());
    }
    Ok((inner, tail))
}

fn edit_flow(name: &str, indent: usize, rest: &str, path: &str) -> Result<String, String> {
    let (inner, tail) = split_flow(rest)?;
    let (mut items, mut replaced) = (Vec::new(), false);
    for item in inner.split(',').map(str::trim).filter(|i| !i.is_empty()) {
        let (key, _) = key_value(item).ok_or("an item is not a plain `key: value`")?;
        match classify(key)? {
            Action::Source => {
                items.push(format!("source: {path}"));
                replaced = true;
            }
            Action::Keep => items.push(item.to_string()),
            Action::Drop => {}
        }
    }
    if !replaced {
        return Err("the body has no `generate:`".into());
    }
    let comment = if tail.is_empty() {
        String::new()
    } else {
        format!("  {tail}")
    };
    Ok(format!(
        "{}{name}: {{ {} }}{comment}",
        " ".repeat(indent),
        items.join(", ")
    ))
}

/// Edits the entry `name` under `bodies:`. `Ok(None)` means the file has no `bodies:` section.
pub(super) fn edit_entry(
    lines: &[String],
    name: &str,
    path: &str,
) -> Result<Option<Vec<String>>, String> {
    let Some(section) = bodies_section(lines) else {
        return Ok(None);
    };
    let (i, indent) =
        entry(lines, &section, name).ok_or_else(|| format!("body `{name}` is not in `bodies:`"))?;
    let (_, rest) = key_value(&lines[i]).ok_or("the entry line is not a plain `key: value`")?;
    let mut out = lines[..i].to_vec();
    let end = if rest.starts_with('{') {
        out.push(edit_flow(name, indent, rest, path)?);
        i + 1
    } else if rest.is_empty() || rest.starts_with('#') {
        let end = block_end(lines, i, indent, section.end);
        out.push(lines[i].clone());
        out.extend(edit_block(&lines[i + 1..end], path)?);
        end
    } else {
        return Err("the entry value is neither a block nor a `{ }` mapping".into());
    };
    out.extend_from_slice(&lines[end..]);
    Ok(Some(out))
}
