//! Build runs started from the studio: one child process at a time, logs streamed to clients.

use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;

use serde::Serialize;

use super::hub::ClientHub;
pub(super) use super::run_status::Status;

const MAX_LINES: usize = 2000;
const MAX_RUNS: usize = 10;

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

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
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
        let id = self.register(&label)?;
        let runs = self.clone();
        let hub = hub.clone();
        thread::spawn(move || {
            let code = runs.execute(&hub, id, command);
            runs.finish(&hub, id, Status::from_code(code));
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

    fn execute(&self, hub: &ClientHub, id: u64, mut command: Command) -> Option<i32> {
        command
            .env("NO_COLOR", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(err) => {
                self.log(hub, id, &format!("cannot start the command: {err}"));
                return None;
            }
        };
        let readers: Vec<_> = [
            child.stdout.take().map(|s| self.pump(hub, id, s)),
            child.stderr.take().map(|s| self.pump(hub, id, s)),
        ]
        .into_iter()
        .flatten()
        .collect();
        let status = child.wait().ok();
        readers.into_iter().for_each(|reader| {
            let _ = reader.join();
        });
        status.and_then(|s| s.code())
    }

    fn pump(
        &self,
        hub: &ClientHub,
        id: u64,
        stream: impl Read + Send + 'static,
    ) -> thread::JoinHandle<()> {
        let (runs, hub) = (self.clone(), hub.clone());
        thread::spawn(move || {
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                runs.log(&hub, id, &line);
            }
        })
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
        if let Some(run) = lock(&self.list).iter_mut().find(|run| run.id == id) {
            run.status = status;
        }
        hub.broadcast(
            &serde_json::json!({ "event": "run", "run": id, "status": status }).to_string(),
        );
    }
}
