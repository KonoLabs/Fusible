//! Child-process plumbing for the git runner: environment, output readers, deadline.

use std::io::Read;
use std::process::{Child, Command, ExitStatus};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::git_run::Unrunnable;

const POLL: Duration = Duration::from_millis(20);

/// Sets the environment that keeps git from asking questions or opening an editor.
pub(super) fn quiet(command: &mut Command) {
    command
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never")
        .env("GIT_EDITOR", "true")
        .env("GIT_SEQUENCE_EDITOR", "true")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "core.quotepath")
        .env("GIT_CONFIG_VALUE_0", "off")
        .env("LC_ALL", "C");
    if std::env::var_os("GIT_SSH_COMMAND").is_none() {
        command.env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes");
    }
}

/// Reads a stream to its end on its own thread, so a full pipe never stalls the child.
pub(super) fn drain(stream: Option<impl Read + Send + 'static>) -> JoinHandle<String> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut stream) = stream {
            let _ = stream.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

/// Joins a reader started by [`drain`].
pub(super) fn collect(reader: JoinHandle<String>) -> String {
    reader.join().unwrap_or_default()
}

/// Waits for `child` to exit, and kills it when `timeout` passes first.
pub(super) fn wait(child: &mut Child, timeout: Duration) -> Result<ExitStatus, Unrunnable> {
    let deadline = Instant::now() + timeout;
    loop {
        let polled = child
            .try_wait()
            .map_err(|e| Unrunnable::Io(e.to_string()))?;
        match polled {
            Some(status) => return Ok(status),
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(Unrunnable::TimedOut);
            }
            None => thread::sleep(POLL),
        }
    }
}
