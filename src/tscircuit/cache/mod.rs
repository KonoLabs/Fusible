//! On-disk cache of tscircuit outputs, keyed by a fingerprint of everything a build reads.

mod fingerprint;
mod hasher;
mod identity;

use std::path::{Path, PathBuf};

use super::RunnerKind;
use fingerprint::fingerprint;
use identity::{disabled, identity};

const KEEP_PER_KIND: usize = 6;

/// Cache entries of one workspace state, stored under `<workspace>/.fusible/cache`.
pub struct Cache {
    dir: PathBuf,
    key: String,
}

impl Cache {
    /// Opens the cache for `entry` inside `root`, or `None` when caching is off or unsafe.
    pub fn open(kind: &RunnerKind, root: &Path, entry: &Path) -> Option<Self> {
        if disabled() {
            return None;
        }
        let salt = format!(
            "{}|{}|{}",
            env!("CARGO_PKG_VERSION"),
            identity(kind),
            entry.display()
        );
        let key = fingerprint(root, &salt)?;
        Some(Self {
            dir: root.join(".fusible").join("cache"),
            key,
        })
    }

    /// Short form of the key for messages.
    pub fn short_key(&self) -> &str {
        &self.key[..8]
    }

    fn path(&self, kind: &str) -> PathBuf {
        self.dir.join(format!("{kind}-{}", self.key))
    }

    /// Returns the stored file for `kind` when this exact workspace state was seen before.
    pub fn get(&self, kind: &str) -> Option<PathBuf> {
        Some(self.path(kind)).filter(|path| path.is_file())
    }

    /// Copies the stored `kind` entry to `dest` and reports whether there was one.
    pub fn restore(&self, kind: &str, dest: &Path) -> bool {
        let Some(hit) = self.get(kind) else {
            return false;
        };
        eprintln!(
            "⚡ {kind} unchanged ({}), reusing the cache",
            self.short_key()
        );
        std::fs::copy(hit, dest).is_ok()
    }

    /// Stores a copy of `source` as the `kind` entry; a failure only costs a later rebuild.
    pub fn put(&self, kind: &str, source: &Path) {
        if std::fs::create_dir_all(&self.dir).is_err() {
            return;
        }
        let target = self.path(kind);
        let partial = target.with_extension(format!("{}.part", std::process::id()));
        if std::fs::copy(source, &partial).is_err() || std::fs::rename(&partial, &target).is_err() {
            let _ = std::fs::remove_file(&partial);
            return;
        }
        self.prune(kind);
    }

    fn prune(&self, kind: &str) {
        let prefix = format!("{kind}-");
        let mut entries: Vec<_> = std::fs::read_dir(&self.dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
            .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
            .collect();
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.0));
        for (_, path) in entries.into_iter().skip(KEEP_PER_KIND) {
            let _ = std::fs::remove_file(path);
        }
    }
}
