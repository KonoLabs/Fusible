//! Runs one child process for the studio: lossy line output, a deadline, and bounded joins.

use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Longest stored line in bytes; the rest of a longer line is dropped.
pub(super) const MAX_LINE: usize = 4096;
const POLL: Duration = Duration::from_millis(20);
const READER_GRACE: Duration = Duration::from_secs(2);

/// Drops the rest of the current line.
fn skip_rest(reader: &mut impl BufRead) -> std::io::Result<()> {
    loop {
        let avail = reader.fill_buf()?;
        let end = avail.iter().position(|&b| b == b'\n');
        let used = end.map_or(avail.len(), |i| i + 1);
        reader.consume(used);
        if end.is_some() || used == 0 {
            return Ok(());
        }
    }
}

/// Reads one line into `buf` as raw bytes, dropping whatever exceeds [`MAX_LINE`].
///
/// Returns `false` at the end of the stream. Bytes that are not UTF-8 are not an error here.
fn read_line(reader: &mut impl BufRead, buf: &mut Vec<u8>) -> std::io::Result<bool> {
    buf.clear();
    let read = reader
        .by_ref()
        .take(MAX_LINE as u64)
        .read_until(b'\n', buf)?;
    if buf.last() != Some(&b'\n') && read >= MAX_LINE {
        skip_rest(reader)?;
    }
    Ok(read > 0)
}

pub(super) fn pump(
    stream: impl Read + Send + 'static,
    on_line: impl Fn(&str) + Send + 'static,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let (mut reader, mut buf) = (BufReader::new(stream), Vec::new());
        while let Ok(true) = read_line(&mut reader, &mut buf) {
            let line = String::from_utf8_lossy(&buf);
            on_line(line.trim_end_matches(['\n', '\r']));
        }
    })
}

/// Waits for `child`; `Some(code)` when it exited in time, `None` when it was killed.
fn wait_with_deadline(child: &mut Child, timeout: Duration) -> Option<Option<i32>> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status.code()),
            Ok(None) if start.elapsed() < timeout => thread::sleep(POLL),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}

/// Joins `readers` for at most [`READER_GRACE`]; a reader kept open by a grandchild is left behind.
fn join_briefly(readers: Vec<JoinHandle<()>>) {
    let start = Instant::now();
    while readers.iter().any(|r| !r.is_finished()) && start.elapsed() < READER_GRACE {
        thread::sleep(POLL);
    }
    for reader in readers.into_iter().filter(JoinHandle::is_finished) {
        let _ = reader.join();
    }
}

/// Runs `command` and reports every output line to `on_line`; returns its exit code.
///
/// `None` means the command could not start, was killed after `timeout`, or has no exit code.
pub(super) fn execute(
    mut command: Command,
    timeout: Duration,
    on_line: impl Fn(&str) + Clone + Send + 'static,
) -> Option<i32> {
    command
        .env("NO_COLOR", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            on_line(&format!("cannot start the command: {err}"));
            return None;
        }
    };
    let readers: Vec<_> = [
        child.stdout.take().map(|s| pump(s, on_line.clone())),
        child.stderr.take().map(|s| pump(s, on_line.clone())),
    ]
    .into_iter()
    .flatten()
    .collect();
    let code = wait_with_deadline(&mut child, timeout);
    join_briefly(readers);
    if code.is_none() {
        let seconds = timeout.as_secs();
        on_line(&format!("stopped after {seconds} s without finishing"));
    }
    code.flatten()
}
