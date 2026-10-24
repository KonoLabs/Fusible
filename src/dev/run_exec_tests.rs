use std::io::Cursor;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::run_exec::{execute, pump, MAX_LINE};
use crate::test_shell::{shell, sleeping};

fn collect(command: Command, timeout: Duration) -> (Option<i32>, Vec<String>, Duration) {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let sink = lines.clone();
    let start = Instant::now();
    let code = execute(command, timeout, move |line| {
        sink.lock().unwrap().push(line.to_string());
    });
    let lines = lines.lock().unwrap().clone();
    (code, lines, start.elapsed())
}

fn pumped(bytes: Vec<u8>) -> Vec<String> {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let sink = lines.clone();
    pump(Cursor::new(bytes), move |line| {
        sink.lock().unwrap().push(line.to_string());
    })
    .join()
    .unwrap();
    let lines = lines.lock().unwrap().clone();
    lines
}

#[test]
fn non_utf8_output_is_kept_lossily_and_does_not_stop_the_drain() {
    let mut bytes = b"\xff\xfe bad\ncrlf\r\n".to_vec();
    bytes.extend(b"filler\n".repeat(60_000));
    bytes.extend(b"done\n");
    let lines = pumped(bytes);
    assert_eq!(lines.first().unwrap(), "\u{fffd}\u{fffd} bad");
    assert_eq!(lines[1], "crlf");
    assert_eq!(lines.last().unwrap(), "done");
    assert_eq!(lines.len(), 60_003);
}

#[test]
fn a_line_without_a_newline_is_cut_and_the_stream_goes_on() {
    let mut bytes = vec![b'a'; 100_000];
    bytes.extend(b"\nnext\n");
    let lines = pumped(bytes);
    assert_eq!(lines[0].len(), MAX_LINE);
    assert_eq!(lines[1], "next");
    assert_eq!(lines.len(), 2);
}

#[test]
fn output_from_a_child_process_is_collected() {
    let (code, lines, _) = collect(
        shell("echo hello; exit 3", "echo hello& exit 3"),
        Duration::from_secs(20),
    );
    assert_eq!(code, Some(3));
    assert_eq!(lines, ["hello"]);
}

#[test]
fn a_command_that_runs_too_long_is_killed_and_reported() {
    let (code, lines, took) = collect(sleeping(30), Duration::from_millis(300));
    assert_eq!(code, None);
    assert!(took < Duration::from_secs(5), "took {took:?}");
    assert!(lines.last().unwrap().starts_with("stopped after"));
}

#[test]
fn a_grandchild_holding_the_pipe_cannot_wedge_the_run() {
    let (code, lines, took) = collect(
        shell(
            "sleep 4 & echo started",
            "start /B ping -n 5 127.0.0.1 >nul & echo started",
        ),
        Duration::from_secs(20),
    );
    assert_eq!(code, Some(0));
    assert!(took < Duration::from_secs(3), "took {took:?}");
    assert_eq!(lines, ["started"]);
}

#[test]
fn a_missing_program_is_reported_without_a_code() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let sink = lines.clone();
    let code = execute(
        Command::new("/nonexistent/fusible"),
        Duration::from_secs(1),
        move |line| sink.lock().unwrap().push(line.to_string()),
    );
    assert_eq!(code, None);
    assert!(lines.lock().unwrap()[0].contains("cannot start"));
}
