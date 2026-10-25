use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};

use tauri_plugin_shell::process::CommandChild;

use crate::log::Log;

/// Handle to the running studio sidecar and the output of the last failed start.
///
/// The child holds the sidecar's stdin open, so the sidecar exits on its own if the shell dies.
#[derive(Default)]
pub struct Studio {
    child: Mutex<Option<CommandChild>>,
    log: Arc<Mutex<Log>>,
    error: Mutex<Option<String>>,
    granted: Mutex<HashSet<String>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Studio {
    /// Kills the running sidecar, if any.
    pub fn stop(&self) {
        if let Some(child) = lock(&self.child).take() {
            let _ = child.kill();
        }
    }

    /// Stops any running sidecar and clears the log and the last error.
    pub fn reset(&self) {
        self.stop();
        lock(&self.log).clear();
        *lock(&self.error) = None;
    }

    /// Keeps the child so it can be killed later.
    pub fn hold(&self, child: CommandChild) {
        *lock(&self.child) = Some(child);
    }

    /// The shared stderr log.
    pub fn log(&self) -> Arc<Mutex<Log>> {
        Arc::clone(&self.log)
    }

    /// Records the message of a failed start and returns it.
    pub fn fail(&self, message: String) -> String {
        self.stop();
        *lock(&self.error) = Some(message.clone());
        message
    }

    /// Records a granted URL pattern and tells whether it is new.
    pub fn mark_granted(&self, pattern: &str) -> bool {
        lock(&self.granted).insert(pattern.to_string())
    }

    /// The message of the last failed start.
    pub fn last_error(&self) -> Option<String> {
        lock(&self.error).clone()
    }
}
