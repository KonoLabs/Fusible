//! Shared helpers of the authored body tests.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::authored::Failure;
use crate::config::FusibleConfig;

pub(super) struct Dir(pub PathBuf);

impl Dir {
    pub(super) fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let name = format!("fusible_authored_{}_{id}", std::process::id());
        let path = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    pub(super) fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    pub(super) fn cache(&self) -> PathBuf {
        self.0.join("cache")
    }

    pub(super) fn glbs(&self) -> usize {
        let entries = std::fs::read_dir(self.cache())
            .into_iter()
            .flatten()
            .flatten();
        let is_glb = |e: &std::fs::DirEntry| e.path().extension().is_some_and(|x| x == "glb");
        entries.filter(is_glb).count()
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A manifest with the authored `bodies` (name and absolute source) and a generated body `lid`.
pub(super) fn manifest(bodies: &[(&str, &Path)]) -> FusibleConfig {
    let lines: Vec<String> = bodies
        .iter()
        .map(|(name, path)| format!("  {name}: {{source: \"{}\"}}\n", path.display()))
        .collect();
    let yaml = format!(
        "bodies:\n{}  lid: {{generate: enclosure}}\n",
        lines.concat()
    );
    FusibleConfig::from_str(&yaml).unwrap()
}

pub(super) fn no_convert(_: &Path, _: &str, _: &Path) -> Result<(), String> {
    panic!("conversion must not run");
}

pub(super) fn failure(result: Result<Vec<u8>, Failure>) -> (u16, String) {
    let failure = result.expect_err("expected a failure");
    (failure.status, failure.message)
}
