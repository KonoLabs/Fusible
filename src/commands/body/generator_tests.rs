use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::generator::wait_or_kill;

fn spawn(script: &str) -> std::process::Child {
    Command::new("sh")
        .args(["-c", script])
        .stdout(Stdio::null())
        .spawn()
        .unwrap()
}

#[test]
fn a_script_that_runs_too_long_is_killed() {
    let mut child = spawn("sleep 30");
    let start = Instant::now();
    assert!(wait_or_kill(&mut child, Duration::from_millis(200)).is_none());
    assert!(start.elapsed() < Duration::from_secs(5));
    assert!(child.try_wait().unwrap().is_some(), "the child was reaped");
}

#[test]
fn a_script_that_finishes_in_time_reports_its_status() {
    let mut child = spawn("exit 3");
    let status = wait_or_kill(&mut child, Duration::from_secs(10)).unwrap();
    assert_eq!(status.code(), Some(3));
}
