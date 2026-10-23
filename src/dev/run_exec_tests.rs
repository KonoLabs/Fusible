use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::run_exec::{execute, MAX_LINE};

fn shell(script: &str) -> Command {
    let mut command = Command::new("sh");
    command.args(["-c", script]);
    command
}

fn run(script: &str, timeout: Duration) -> (Option<i32>, Vec<String>, Duration) {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let sink = lines.clone();
    let start = Instant::now();
    let code = execute(shell(script), timeout, move |line| {
        sink.lock().unwrap().push(line.to_string());
    });
    let lines = lines.lock().unwrap().clone();
    (code, lines, start.elapsed())
}

#[test]
fn non_utf8_output_is_kept_lossily_and_does_not_stop_the_drain() {
    let script = r"printf '\377\376 bad\n'; yes filler | head -n 60000; echo done";
    let (code, lines, took) = run(script, Duration::from_secs(20));
    assert_eq!(code, Some(0));
    assert!(took < Duration::from_secs(15), "took {took:?}");
    assert_eq!(lines.first().unwrap(), "\u{fffd}\u{fffd} bad");
    assert_eq!(lines.last().unwrap(), "done");
    assert_eq!(lines.len(), 60_002);
}

#[test]
fn a_line_without_a_newline_is_cut_and_the_stream_goes_on() {
    let script = "head -c 100000 /dev/zero | tr '\\0' a; echo; echo next";
    let (code, lines, _) = run(script, Duration::from_secs(20));
    assert_eq!(code, Some(0));
    assert_eq!(lines[0].len(), MAX_LINE);
    assert_eq!(lines[1], "next");
    assert_eq!(lines.len(), 2);
}

#[test]
fn a_command_that_runs_too_long_is_killed_and_reported() {
    let (code, lines, took) = run("sleep 30", Duration::from_millis(300));
    assert_eq!(code, None);
    assert!(took < Duration::from_secs(5), "took {took:?}");
    assert!(lines.last().unwrap().starts_with("stopped after"));
}

#[test]
fn a_grandchild_holding_the_pipe_cannot_wedge_the_run() {
    let (code, lines, took) = run("sleep 4 & echo started", Duration::from_secs(20));
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
