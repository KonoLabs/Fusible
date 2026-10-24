//! The recent saves and the authors behind conflicting files.

use std::collections::BTreeMap;

use serde::Serialize;

use super::git_run::Git;

const SAVES: &str = "-20";
const FILES_SHOWN: usize = 50;

/// One entry of the history, as the panel lists it.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct Save {
    pub id: String,
    pub author: String,
    pub when: i64,
    pub message: String,
    pub files: Vec<String>,
}

fn parse_save(block: &str) -> Option<Save> {
    let mut lines = block.lines();
    let mut head = lines.next()?.split('\x1f');
    Some(Save {
        id: head.next()?.into(),
        author: head.next()?.into(),
        when: head.next()?.parse().ok()?,
        message: head.next().unwrap_or_default().into(),
        files: lines
            .filter(|l| !l.is_empty())
            .take(FILES_SHOWN)
            .map(String::from)
            .collect(),
    })
}

/// The last 20 saves on the current line of work, newest first.
pub(super) fn last_saves(git: &Git) -> Vec<Save> {
    let format = "--format=%x1e%H%x1f%an%x1f%at%x1f%s";
    let raw = git
        .text(&["log", SAVES, format, "--name-only"])
        .unwrap_or_default();
    raw.split('\x1e').filter_map(parse_save).collect()
}

/// Who last changed each conflicting file on the side that came from the remote.
pub(super) fn conflict_authors(git: &Git, paths: &[String]) -> BTreeMap<String, String> {
    paths
        .iter()
        .filter_map(|path| {
            let args = ["log", "-1", "--format=%an", "HEAD", "--", path.as_str()];
            Some((path.clone(), git.text(&args).filter(|a| !a.is_empty())?))
        })
        .collect()
}
