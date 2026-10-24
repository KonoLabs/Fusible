use std::process::Command;
use std::time::{Duration, Instant};

use super::hub::ClientHub;
use super::runs::{Runs, Status};

use crate::test_shell::{shell, sleeping};

fn exit(code: i32) -> Command {
    shell(&format!("exit {code}"), &format!("exit {code}"))
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
    let rx = hub.subscribe(Vec::new);
    runs.start(
        &hub,
        "demo".into(),
        shell(
            "echo out; echo err >&2; exit 1",
            "echo out& >&2 echo err& exit 1",
        ),
    )
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
    runs.start(&hub, "slow".into(), sleeping(1)).unwrap();
    assert!(runs.start(&hub, "second".into(), exit(0)).is_err());
    wait_until_done(&runs);
    assert!(runs.start(&hub, "third".into(), exit(0)).is_ok());
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
fn run_events_carry_the_label_when_the_run_starts_and_when_it_ends() {
    let runs = Runs::default();
    let hub = ClientHub::default();
    let rx = hub.subscribe(Vec::new);
    runs.start(&hub, "check main".into(), exit(0)).unwrap();
    wait_until_done(&runs);
    let events: Vec<String> = rx
        .try_iter()
        .map(|m| String::from_utf8(m).unwrap())
        .collect();
    let first = &events[0];
    assert!(first.contains(r#""event":"run""#) && first.contains(r#""label":"check main""#));
    assert!(first.contains(r#""status":"running""#));
    let last = events.last().unwrap();
    assert!(last.contains(r#""label":"check main""#) && last.contains(r#""status":"ok""#));
}

#[test]
fn a_run_that_hangs_is_killed_and_frees_the_slot() {
    let runs = Runs::default();
    let hub = ClientHub::default();
    runs.start_with(
        &hub,
        "hang".into(),
        sleeping(30),
        Duration::from_millis(200),
    )
    .unwrap();
    let snapshot = wait_until_done(&runs);
    assert_eq!(snapshot[0]["status"], "error");
    assert!(runs.start(&hub, "next".into(), exit(0)).is_ok());
}

#[test]
fn exit_codes_follow_the_fuse_contract() {
    assert_eq!(Status::from_code(Some(0)), Status::Ok);
    assert_eq!(Status::from_code(Some(1)), Status::FuseBlown);
    assert_eq!(Status::from_code(Some(2)), Status::Error);
    assert_eq!(Status::from_code(None), Status::Error);
}
