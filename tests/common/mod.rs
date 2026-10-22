//! Shared helpers for the subcommand integration tests.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Returns a command for the compiled `fusible` binary.
pub fn fusible() -> Command {
    Command::new(env!("CARGO_BIN_EXE_fusible"))
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
