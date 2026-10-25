//! Shared helpers for the subcommand integration tests.

#![allow(dead_code)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Returns a command for the compiled `fusible` binary.
pub fn fusible() -> Command {
    Command::new(env!("CARGO_BIN_EXE_fusible"))
}

/// Finds `bun` on the real `PATH`.
pub fn bun() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join("bun"))
        .find(|candidate| candidate.is_file())
}

/// Finds `bun`, or panics when `FUSIBLE_REQUIRE_BUN=1` demands it, so CI cannot skip silently.
pub fn require_bun() -> Option<PathBuf> {
    let found = bun();
    if found.is_none() && std::env::var("FUSIBLE_REQUIRE_BUN").as_deref() == Ok("1") {
        panic!("FUSIBLE_REQUIRE_BUN=1 but bun is not on PATH");
    }
    found
}

/// A unique temporary directory removed on drop.
pub struct TestDir {
    pub path: std::path::PathBuf,
}

impl TestDir {
    /// Creates an empty directory unique to this process and call.
    pub fn new(name: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let path = std::env::temp_dir().join(format!("fusible_integration_{pid}_{id}_{name}"));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("failed to create test dir");
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
