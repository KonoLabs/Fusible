//! Build runs started from the studio: one child process at a time, logs streamed to clients.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::Serialize;

use super::hub::ClientHub;
use super::limiter::lock;
use super::run_exec::execute;
pub(super) use super::run_status::Status;

const MAX_LINES: usize = 2000;
const MAX_RUNS: usize = 10;
const RUN_TIMEOUT: Duration = Duration::from_secs(30 * 60);

#[derive(Debug, Clone, Serialize)]
pub(super) struct Run {
    pub id: u64,
    pub label: String,
    pub status: Status,
    pub lines: Vec<String>,
}

/// A run could not start because another one is still going.
#[derive(Debug)]
pub(super) struct Busy;

/// The recent runs and the counter that numbers them.
#[derive(Clone, Default)]
pub(super) struct Runs {
    list: Arc<Mutex<Vec<Run>>>,
    next: Arc<AtomicU64>,
}

/// Frees the run slot when the run thread ends, even by panicking.
struct Finisher {
    runs: Runs,
    hub: ClientHub,
    id: u64,
    status: Status,
}

impl Finisher {
    fn complete(mut self, status: Status) {
        self.status = status;
    }
}

impl Drop for Finisher {
    fn drop(&mut self) {
        self.runs.finish(&self.hub, self.id, self.status);
    }
}

impl Runs {
    /// Serializes the recent runs, oldest first, for `GET /api/runs`.
    pub(super) fn snapshot(&self) -> String {
        serde_json::to_string(&*lock(&self.list)).unwrap_or_else(|_| "[]".into())
    }

    /// Starts `command` as run `label`, or fails with [`Busy`] while a run is going.
    pub(super) fn start(
        &self,
        hub: &ClientHub,
        label: String,
        command: Command,
    ) -> Result<u64, Busy> {
        self.start_with(hub, label, command, RUN_TIMEOUT)
    }

    /// Like [`Runs::start`], killing the command when it runs longer than `timeout`.
    pub(super) fn start_with(
        &self,
        hub: &ClientHub,
        label: String,
        command: Command,
        timeout: Duration,
    ) -> Result<u64, Busy> {
        let id = self.register(&label)?;
        let event = serde_json::json!({ "event": "run", "run": id, "label": label, "status": Status::Running });
        hub.broadcast(&event.to_string());
        let (runs, hub) = (self.clone(), hub.clone());
        thread::spawn(move || {
            let finisher = Finisher {
                runs: runs.clone(),
                hub: hub.clone(),
                id,
                status: Status::Error,
            };
            let sink = move |line: &str| runs.log(&hub, id, line);
            let code = execute(command, timeout, sink);
            finisher.complete(Status::from_code(code));
        });
        Ok(id)
    }

    fn register(&self, label: &str) -> Result<u64, Busy> {
        let mut list = lock(&self.list);
        if list.iter().any(|run| run.status == Status::Running) {
            return Err(Busy);
        }
        let id = self.next.fetch_add(1, Ordering::SeqCst) + 1;
        let (label, lines) = (label.to_string(), Vec::new());
        list.push(Run {
            id,
            label,
            status: Status::Running,
            lines,
        });
        let excess = list.len().saturating_sub(MAX_RUNS);
        list.drain(..excess);
        Ok(id)
    }

    fn log(&self, hub: &ClientHub, id: u64, line: &str) {
        if let Some(run) = lock(&self.list).iter_mut().find(|run| run.id == id) {
            if run.lines.len() < MAX_LINES {
                run.lines.push(line.to_string());
            }
        }
        hub.broadcast(&serde_json::json!({ "event": "log", "run": id, "line": line }).to_string());
    }

    fn finish(&self, hub: &ClientHub, id: u64, status: Status) {
        let mut list = lock(&self.list);
        let Some(run) = list.iter_mut().find(|run| run.id == id) else {
            return;
        };
        run.status = status;
        let label = run.label.clone();
        drop(list);
        let event =
            serde_json::json!({ "event": "run", "run": id, "label": label, "status": status });
        hub.broadcast(&event.to_string());
    }
}
