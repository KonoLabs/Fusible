use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::kicad_status::KicadStates;
use super::kicad_sync::{evaluate, sync_slot};
use super::kicad_watch::start_kicad_watcher;
use super::state::State;
use crate::commands::target::BoardTarget;

const TSX: &str = r#"<board>
  <resistor name="R1" pcbX={0} pcbY={0} />
  <resistor name="R2" pcbX={3} pcbY={0} />
  <resistor name="R3" pcbX={6} pcbY={0} />
</board>
"#;

const CIRCUIT: &str = r#"[
  {"type":"source_component","source_component_id":"a","name":"R1"},
  {"type":"source_component","source_component_id":"b","name":"R2"},
  {"type":"source_component","source_component_id":"c","name":"R3"},
  {"type":"pcb_component","source_component_id":"a","center":{"x":0,"y":0},"rotation":0,"layer":"top"},
  {"type":"pcb_component","source_component_id":"b","center":{"x":3,"y":0},"rotation":0,"layer":"top"},
  {"type":"pcb_component","source_component_id":"c","center":{"x":6,"y":0},"rotation":0,"layer":"top"}]"#;

fn project(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fusible_kicad_dev_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_pcb(dir: &Path, third_x: f64, extra: &str) -> PathBuf {
    let part = |name: &str, x: f64| {
        format!(r#"(footprint "R" (layer "F.Cu") (at {x} 40) (property "Reference" "{name}"))"#)
    };
    let path = dir.join("board.kicad_pcb");
    let body = format!(
        "(kicad_pcb {} {} {} {extra})",
        part("R1", 100.0),
        part("R2", 103.0),
        part("R3", third_x)
    );
    std::fs::write(&path, body).unwrap();
    path
}

fn state_for(dir: &Path) -> State {
    let board = BoardTarget {
        name: Some("main".into()),
        entry: dir.join("index.tsx"),
        ..Default::default()
    };
    let mut state = State::new(&[board]);
    state.kicad = KicadStates::with_backup(dir.join(".fusible/backup"));
    state.slots[0].set_source(Some(CIRCUIT));
    state
}

fn wait_for(what: &str, mut done: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(8);
    while !done() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn saving_the_kicad_file_syncs_once_and_does_not_loop() {
    let dir = project("watch");
    write_pcb(&dir, 106.0, "");
    let tsx = dir.join("index.tsx");
    std::fs::write(&tsx, TSX).unwrap();
    let state = state_for(&dir);
    let _watcher = start_kicad_watcher(&state.slots[0], &state).expect("a board file is watched");
    write_pcb(&dir, 130.0, "");
    wait_for("the sync", || {
        std::fs::read_to_string(&tsx).is_ok_and(|text| text.contains(r#"name="R3" pcbX={30}"#))
    });
    let written = std::fs::metadata(&tsx).unwrap().modified().unwrap();
    std::thread::sleep(Duration::from_millis(2000));
    assert_eq!(
        std::fs::metadata(&tsx).unwrap().modified().unwrap(),
        written
    );
    let backups = std::fs::read_dir(dir.join(".fusible/backup")).unwrap();
    assert_eq!(backups.count(), 1);
}

#[test]
fn a_difference_is_reported_to_the_studio_and_nothing_is_written() {
    let dir = project("notice");
    write_pcb(
        &dir,
        106.0,
        r#"(footprint "R" (layer "F.Cu") (at 160 40) (property "Reference" "R9"))"#,
    );
    let state = state_for(&dir);
    let events = state.hub.subscribe(Vec::new);
    evaluate(&state.slots[0], &state);
    let message = String::from_utf8(events.recv().unwrap()).unwrap();
    assert!(message.contains(r#""event":"sync""#), "{message}");
    assert!(message.contains("R9 added in KiCad"), "{message}");
    assert!(message.contains(r#""state":"differs""#), "{message}");
    sync_slot(&state.slots[0], &state);
    assert!(!dir.join(".fusible").exists());
}

#[test]
fn unparseable_circuit_json_is_an_error_status() {
    let dir = project("broken");
    write_pcb(&dir, 106.0, "");
    let state = state_for(&dir);
    state.slots[0].set_source(Some(r#"[{"type": "source_component""#));
    evaluate(&state.slots[0], &state);
    assert!(state.kicad.json().contains(r#""state":"error""#));
}
