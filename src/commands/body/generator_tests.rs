use std::process::Stdio;
use std::time::{Duration, Instant};

use super::generator::wait_or_kill;

fn spawn(unix: &str, windows: &str) -> std::process::Child {
    crate::test_shell::shell(unix, windows)
        .stdout(Stdio::null())
        .spawn()
        .unwrap()
}

#[test]
fn a_script_that_runs_too_long_is_killed() {
    let mut child = spawn("sleep 30", "ping -n 31 127.0.0.1 >nul");
    let start = Instant::now();
    assert!(wait_or_kill(&mut child, Duration::from_millis(200)).is_none());
    assert!(start.elapsed() < Duration::from_secs(5));
    assert!(child.try_wait().unwrap().is_some(), "the child was reaped");
}

#[test]
fn a_script_that_finishes_in_time_reports_its_status() {
    let mut child = spawn("exit 3", "exit 3");
    let status = wait_or_kill(&mut child, Duration::from_secs(10)).unwrap();
    assert_eq!(status.code(), Some(3));
}
