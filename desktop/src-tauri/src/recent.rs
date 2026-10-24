use crate::model::{Recent, RECENT_LIMIT};

/// Puts `path` first with the given timestamp, dropping older duplicates and capping the list.
pub fn push(list: Vec<Recent>, path: &str, now: &str) -> Vec<Recent> {
    let head = Recent {
        path: path.into(),
        last_opened: now.into(),
    };
    let rest = list.into_iter().filter(|entry| entry.path != path);
    std::iter::once(head)
        .chain(rest)
        .take(RECENT_LIMIT)
        .collect()
}

/// Removes `path` from the list.
pub fn remove(list: Vec<Recent>, path: &str) -> Vec<Recent> {
    list.into_iter()
        .filter(|entry| entry.path != path)
        .collect()
}
