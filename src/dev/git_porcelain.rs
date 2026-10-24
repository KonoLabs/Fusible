//! Reads `git status --porcelain=v2 -z` into plain changes.

use serde::Serialize;

/// One changed file, with a plain kind: `added`, `modified`, `deleted`, `renamed` or `conflict`.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct Change {
    pub path: String,
    pub kind: &'static str,
}

/// The branch line and the changed files of one status output.
#[derive(Default)]
pub(super) struct Porcelain {
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub changes: Vec<Change>,
}

fn kind_of(xy: &str) -> &'static str {
    match (xy.contains('A'), xy.contains('D'), xy.contains('R')) {
        (true, ..) => "added",
        (_, true, _) => "deleted",
        (.., true) => "renamed",
        _ => "modified",
    }
}

fn parse_ab(value: &str, out: &mut Porcelain) {
    let number = |s: &str| s.trim_start_matches(['+', '-']).parse().unwrap_or(0);
    let mut parts = value.split(' ');
    out.ahead = parts.next().map_or(0, number);
    out.behind = parts.next().map_or(0, number);
}

fn parse_header(header: &str, out: &mut Porcelain) {
    match header.split_once(' ') {
        Some(("branch.head", name)) if name != "(detached)" => out.branch = Some(name.into()),
        Some(("branch.upstream", name)) => out.upstream = Some(name.into()),
        Some(("branch.ab", value)) => parse_ab(value, out),
        _ => {}
    }
}

fn parse_entry(entry: &str, fields: usize) -> Option<(&str, &str)> {
    let mut parts = entry.splitn(fields, ' ');
    let xy = parts.nth(1)?;
    Some((xy, parts.last()?))
}

/// Parses the NUL-separated status output.
pub(super) fn parse(raw: &str) -> Porcelain {
    let mut out = Porcelain::default();
    let mut tokens = raw.split('\0').filter(|t| !t.is_empty());
    while let Some(token) = tokens.next() {
        let change = match token.as_bytes()[0] {
            b'#' => {
                parse_header(&token[2..], &mut out);
                None
            }
            b'?' => Some((token[2..].to_string(), "added")),
            b'1' => parse_entry(token, 9).map(|(xy, p)| (p.to_string(), kind_of(xy))),
            b'2' => {
                tokens.next();
                parse_entry(token, 10).map(|(xy, p)| (p.to_string(), kind_of(xy)))
            }
            b'u' => parse_entry(token, 11).map(|(_, p)| (p.to_string(), "conflict")),
            _ => None,
        };
        out.changes
            .extend(change.map(|(path, kind)| Change { path, kind }));
    }
    out
}
