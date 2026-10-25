use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::async_runtime::Receiver;
use tauri::{AppHandle, Runtime};
use tauri_plugin_shell::process::CommandEvent;

use crate::log::Log;
use crate::ready::{self, Ready};
use crate::sidecar;
use crate::studio::Studio;

const READY_TIMEOUT: Duration = Duration::from_secs(30);

type SharedLog = Arc<Mutex<Log>>;

fn push(log: &SharedLog, line: &str) {
    if let Ok(mut log) = log.lock() {
        log.push(line);
    }
}

fn snapshot(log: &SharedLog) -> String {
    log.lock().map(|log| log.text()).unwrap_or_default()
}

async fn wait_ready(rx: &mut Receiver<CommandEvent>, log: &SharedLog) -> Result<Ready, String> {
    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(bytes) => {
                if let Some(found) = ready::parse(&String::from_utf8_lossy(&bytes)) {
                    return Ok(found);
                }
            }
            CommandEvent::Stderr(bytes) => push(log, &String::from_utf8_lossy(&bytes)),
            CommandEvent::Error(message) => push(log, &message),
            CommandEvent::Terminated(payload) => {
                let code = payload
                    .code
                    .map_or("unknown".into(), |code| code.to_string());
                return Err(format!(
                    "fusible exited with code {code} before the studio was ready."
                ));
            }
            _ => {}
        }
    }
    Err("fusible closed its output before the studio was ready.".into())
}

async fn drain(mut rx: Receiver<CommandEvent>, log: SharedLog) {
    while let Some(event) = rx.recv().await {
        if let CommandEvent::Stderr(bytes) = event {
            push(&log, &String::from_utf8_lossy(&bytes));
        }
    }
}

fn describe_failure(reason: &str, log: &SharedLog) -> String {
    let captured = snapshot(log);
    if captured.is_empty() {
        reason.to_string()
    } else {
        format!("{reason}\n\n{captured}")
    }
}

/// Starts `fusible studio --desktop` in `dir` and waits for its ready line.
///
/// Replaces any studio already running. On failure the sidecar is killed and the
/// captured stderr becomes the error and the value of `last_error`.
pub async fn start<R: Runtime>(
    app: &AppHandle<R>,
    studio: &Studio,
    dir: &Path,
    port: u16,
) -> Result<Ready, String> {
    studio.reset();
    let command = sidecar::fusible(app, dir)?.args(["studio", "--desktop", "-p", &port.to_string()]);
    let (mut rx, child) = command
        .spawn()
        .map_err(|err| studio.fail(format!("Could not start fusible: {err}")))?;
    studio.hold(child);
    let log = studio.log();
    let outcome = tokio::time::timeout(READY_TIMEOUT, wait_ready(&mut rx, &log)).await;
    match outcome {
        Ok(Ok(found)) => {
            tauri::async_runtime::spawn(drain(rx, log));
            Ok(found)
        }
        Ok(Err(reason)) => Err(studio.fail(describe_failure(&reason, &log))),
        Err(_) => {
            let reason = "The studio did not start within 30 seconds.";
            Err(studio.fail(describe_failure(reason, &log)))
        }
    }
}
