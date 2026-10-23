use std::process::Command;
use std::time::{Duration, Instant};

use super::hub::ClientHub;
use super::runs::{Runs, Status};

fn shell(script: &str) -> Command {
    let mut command = Command::new("sh");
    command.args(["-c", script]);
    command
}

fn wait_until_done(runs: &Runs) -> serde_json::Value {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let snapshot: serde_json::Value = serde_json::from_str(&runs.snapshot()).unwrap();
        let running = snapshot
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["status"] == "running");
        if !running {
            return snapshot;
        }
        assert!(Instant::now() < deadline, "run did not finish");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn run_collects_both_streams_and_maps_the_exit_code() {
    let runs = Runs::default();
    let hub = ClientHub::default();
    let (tx, rx) = std::sync::mpsc::channel();
    hub.add_client(tx);
    runs.start(&hub, "demo".into(), shell("echo out; echo err >&2; exit 1"))
        .unwrap();
    let snapshot = wait_until_done(&runs);
    assert_eq!(snapshot[0]["status"], "fuse_blown");
    let mut lines: Vec<&str> = snapshot[0]["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect();
    lines.sort_unstable();
    assert_eq!(lines, ["err", "out"]);
    let events: Vec<String> = rx
        .try_iter()
        .map(|m| String::from_utf8(m).unwrap())
        .collect();
    assert!(events
        .iter()
        .any(|e| e.contains(r#""event":"log""#) && e.contains("out")));
    assert!(events.last().unwrap().contains(r#""status":"fuse_blown""#));
}

#[test]
fn only_one_run_goes_at_a_time() {
    let runs = Runs::default();
    let hub = ClientHub::default();
    runs.start(&hub, "slow".into(), shell("sleep 0.3")).unwrap();
    assert!(runs.start(&hub, "second".into(), shell("true")).is_err());
    wait_until_done(&runs);
    assert!(runs.start(&hub, "third".into(), shell("true")).is_ok());
}

#[test]
fn unspawnable_command_is_an_error_run() {
    let runs = Runs::default();
    let hub = ClientHub::default();
    runs.start(&hub, "missing".into(), Command::new("/nonexistent/fusible"))
        .unwrap();
    let snapshot = wait_until_done(&runs);
    assert_eq!(snapshot[0]["status"], "error");
    assert!(snapshot[0]["lines"][0]
        .as_str()
        .unwrap()
        .contains("cannot start"));
}

#[test]
fn exit_codes_follow_the_fuse_contract() {
    assert_eq!(Status::from_code(Some(0)), Status::Ok);
    assert_eq!(Status::from_code(Some(1)), Status::FuseBlown);
    assert_eq!(Status::from_code(Some(2)), Status::Error);
    assert_eq!(Status::from_code(None), Status::Error);
}
