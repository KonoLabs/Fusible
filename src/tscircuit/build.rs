//! `build` invocation: drains output pipes, enforces a timeout, and locates fresh Circuit JSON.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, ExitStatus, Output, Stdio};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime};

use super::process::{extract_output_details, format_command};
use super::{command, output, resolve_hardware_context, Runner, RunnerError};

const CACHE_KIND: &str = "build";

/// A spawned build process with its display string and launch time.
struct SpawnedBuild {
    child: Child,
    command: String,
    started: SystemTime,
}

impl Runner {
    /// Runs the tscircuit build command to produce circuit json.
    pub fn build(&self, file: &Path, out_file: Option<&Path>) -> Result<PathBuf, RunnerError> {
        let cache = self.cache_for(file);
        if let Some(hit) = cache.as_ref().and_then(|c| c.get(CACHE_KIND)) {
            let key = cache.as_ref().map_or("", |c| c.short_key());
            eprintln!("⚡ Sources unchanged ({key}), reusing the cached build");
            return output::deliver(hit, out_file);
        }
        let mut spawned = self.spawn_build(file, true)?;
        let mut out = collect_output(&mut spawned.child, &spawned.command)?;
        let stderr = String::from_utf8_lossy(&out.stderr);
        if !out.status.success() && stderr.contains("unknown option '--ignore-errors'") {
            spawned = self.spawn_build(file, false)?;
            out = collect_output(&mut spawned.child, &spawned.command)?;
        }
        let produced = fresh_circuit_json(file, &out, spawned.started);

        if !out.status.success() && produced.is_err() {
            let details = extract_output_details(&out);
            return Err(RunnerError::CommandFailed {
                command: spawned.command,
                code: out.status.code(),
                stderr: details.clone(),
                help: details,
            });
        }

        let produced_path = produced?;
        if let Some(cache) = &cache {
            cache.put(CACHE_KIND, &produced_path);
        }
        output::deliver(produced_path, out_file)
    }

    /// Spawns the build command with piped output in the hardware workspace.
    fn spawn_build(&self, file: &Path, ignore_errors: bool) -> Result<SpawnedBuild, RunnerError> {
        let (working_dir, target_file) = resolve_hardware_context(file);
        let mut cmd = command::build_command(&self.kind, &target_file, ignore_errors);
        if let Some(ref dir) = working_dir {
            cmd.current_dir(dir);
        }
        cmd.stdin(Stdio::null());
        cmd.env("CI", "1");
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let command = format_command(&cmd);
        let started = SystemTime::now() - Duration::from_secs(2);
        let child = cmd.spawn().map_err(|source| RunnerError::Io {
            command: command.clone(),
            source,
        })?;
        Ok(SpawnedBuild {
            child,
            command,
            started,
        })
    }
}

/// Waits for `child` while draining both pipes, then returns its full output.
fn collect_output(child: &mut Child, command: &str) -> Result<Output, RunnerError> {
    let stdout_handle = drain(child.stdout.take());
    let stderr_handle = drain(child.stderr.take());
    let status = wait_with_timeout(child, command)?;
    Ok(Output {
        status,
        stdout: stdout_handle.join().unwrap_or_default(),
        stderr: stderr_handle.join().unwrap_or_default(),
    })
}

/// Reads `pipe` to the end on a background thread.
fn drain<R: Read + Send + 'static>(pipe: Option<R>) -> JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut buf);
        }
        buf
    })
}

/// Polls `child` until it exits; kills it on timeout or wait failure.
fn wait_with_timeout(child: &mut Child, command: &str) -> Result<ExitStatus, RunnerError> {
    let start = Instant::now();
    let timeout = build_timeout();
    let result = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if start.elapsed() >= timeout => break Err(timeout_error(command, timeout)),
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(source) => {
                break Err(RunnerError::Io {
                    command: command.to_string(),
                    source,
                })
            }
        }
    };
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

/// Builds the error reported when compilation exceeds `timeout`.
fn timeout_error(command: &str, timeout: Duration) -> RunnerError {
    RunnerError::CommandFailed {
        command: command.to_string(),
        code: None,
        stderr: format!("compilation timed out after {} seconds", timeout.as_secs()),
        help: "raise FUSIBLE_BUILD_TIMEOUT_SECS or set `routingDisabled` on <board>".to_string(),
    }
}

/// Locates the produced Circuit JSON and rejects it if it predates `started`.
fn fresh_circuit_json(
    file: &Path,
    out: &Output,
    started: SystemTime,
) -> Result<PathBuf, RunnerError> {
    let stdout_str = String::from_utf8_lossy(&out.stdout);
    let path = output::resolve_circuit_json(file, &stdout_str)?;
    let fresh = std::fs::metadata(&path)
        .and_then(|m| m.modified())
        .is_ok_and(|modified| modified >= started);
    if fresh {
        Ok(path)
    } else {
        Err(RunnerError::OutputNotFound { path })
    }
}

/// Returns the tscircuit build timeout: `FUSIBLE_BUILD_TIMEOUT_SECS` when set to a positive integer, else 120 seconds.
fn build_timeout() -> Duration {
    let secs = std::env::var("FUSIBLE_BUILD_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|s| *s > 0)
        .unwrap_or(120);
    Duration::from_secs(secs)
}
