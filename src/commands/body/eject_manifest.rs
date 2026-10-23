//! Turns a generated body of `fusible.yml` into an authored one, verified after the edit.

use std::path::PathBuf;

use super::eject_edit::edit_entry;
use super::yaml_lines::{block_end, is_skippable};
use crate::config::{BodyConfig, FusibleConfig};

/// A YAML scalar for `path`: plain when it is simple, double-quoted otherwise.
fn scalar(path: &str) -> String {
    let simple = path
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "_./-".contains(c));
    if simple && !path.starts_with('-') {
        return path.to_string();
    }
    serde_json::to_string(path).unwrap_or_else(|_| path.to_string())
}

/// Replaces a legacy `mechanical:` block with a `bodies:` section holding the authored body.
fn from_legacy(lines: &[String], name: &str, path: &str) -> Result<Vec<String>, String> {
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(format!("`{name}` cannot be written as a plain YAML key"));
    }
    let i = lines
        .iter()
        .position(|l| l.starts_with("mechanical:"))
        .ok_or("the file has neither a `bodies:` section nor a `mechanical:` block")?;
    let rest = lines[i]["mechanical:".len()..].trim();
    let end = if rest.starts_with('{') && rest.contains('}') {
        i + 1
    } else if is_skippable(rest) {
        block_end(lines, i, 0, lines.len())
    } else {
        return Err("the `mechanical:` value is neither a block nor a one-line mapping".into());
    };
    let mut out = lines[..i].to_vec();
    let after = &lines[end..];
    let blank = |l: Option<&String>| l.is_some_and(|l| l.trim().is_empty());
    let skip = usize::from(blank(out.last()) && blank(after.first()));
    out.extend_from_slice(&after[skip..]);
    while out.last().is_some_and(|l| l.trim().is_empty()) {
        out.pop();
    }
    out.extend(["".into(), "bodies:".into(), format!("  {name}:")]);
    out.push(format!("    source: {}", scalar(path)));
    Ok(out)
}

/// Checks that only `name` changed, and that it is now authored with its board kept.
fn verify(old: &str, new: &str, name: &str, path: &str) -> Result<(), String> {
    let before = FusibleConfig::from_str(old).map_err(|e| e.to_string())?;
    let after =
        FusibleConfig::from_str(new).map_err(|e| format!("the edit does not parse: {e}"))?;
    let body = before
        .bodies
        .get(name)
        .ok_or("the body is not in the manifest")?;
    let mut expected = before.clone();
    expected.bodies.insert(
        name.to_string(),
        BodyConfig {
            source: Some(PathBuf::from(path)),
            board: body.board.clone(),
            ..Default::default()
        },
    );
    if expected == after {
        return Ok(());
    }
    Err("the edit would change more than the body".into())
}

/// Returns `old` with body `name` rewritten to `source: path`, or the reason it cannot be done safely.
pub(super) fn eject_manifest(old: &str, name: &str, path: &str) -> Result<String, String> {
    if old.contains('\r') {
        return Err("the file uses CRLF line endings".into());
    }
    let lines: Vec<String> = old.lines().map(str::to_string).collect();
    let edited = match edit_entry(&lines, name, &scalar(path))? {
        Some(edited) => edited,
        None => from_legacy(&lines, name, path)?,
    };
    let new = edited.join("\n") + "\n";
    verify(old, &new, name, path)?;
    Ok(new)
}
