use std::cell::{Cell, RefCell};
use std::path::Path;

use super::authored::{authored_glb, cache_key};
use super::authored_support::{failure, manifest, Dir};

#[test]
fn step_and_stl_convert_once_and_reconvert_when_the_file_changes() {
    let dir = Dir::new();
    let step = dir.file("case.step", b"v1");
    let cfg = manifest(&[("case", &step), ("knob", &dir.file("knob.stl", b"s"))]);
    let calls = Cell::new(0);
    let formats = RefCell::new(Vec::new());
    let convert = |_: &Path, format: &str, out: &Path| {
        calls.set(calls.get() + 1);
        formats.borrow_mut().push(format.to_string());
        std::fs::write(out, format!("glb{}", calls.get())).map_err(|e| e.to_string())
    };
    let first = authored_glb(&cfg, "case", &dir.cache(), convert).unwrap();
    assert_eq!(first, b"glb1");
    assert_eq!(
        authored_glb(&cfg, "case", &dir.cache(), convert).unwrap(),
        b"glb1"
    );
    assert_eq!(calls.get(), 1);
    std::fs::write(&step, b"version two").unwrap();
    let again = authored_glb(&cfg, "case", &dir.cache(), convert).unwrap();
    assert_eq!(again, b"glb2");
    assert_eq!(dir.glbs(), 1, "the stale entry of the same file is pruned");
    authored_glb(&cfg, "knob", &dir.cache(), convert).unwrap();
    assert_eq!(*formats.borrow(), ["step", "step", "stl"]);
    assert_eq!(dir.glbs(), 2);
}

#[test]
fn a_failed_conversion_is_a_422_and_caches_nothing() {
    let dir = Dir::new();
    let cfg = manifest(&[("case", &dir.file("case.stp", b"x"))]);
    let broken = |_: &Path, _: &str, _: &Path| Err("bad shape".to_string());
    let (status, message) = failure(authored_glb(&cfg, "case", &dir.cache(), broken));
    assert_eq!(status, 422);
    assert!(message.contains("conversion failed: bad shape"));
    assert_eq!(dir.glbs(), 0);
}

#[test]
fn cache_key_tracks_the_file_state_but_keeps_one_prefix_per_path() {
    let dir = Dir::new();
    let path = dir.file("a.step", b"one");
    let before = cache_key(&path, &std::fs::metadata(&path).unwrap());
    assert_eq!(before, cache_key(&path, &std::fs::metadata(&path).unwrap()));
    std::fs::write(&path, b"three").unwrap();
    let after = cache_key(&path, &std::fs::metadata(&path).unwrap());
    let other = dir.file("b.step", b"three");
    let elsewhere = cache_key(&other, &std::fs::metadata(&other).unwrap());
    let prefix = |key: &str| key.split('-').next().unwrap().to_string();
    assert_ne!(before, after);
    assert_eq!(prefix(&before), prefix(&after));
    assert_ne!(prefix(&after), prefix(&elsewhere));
}

#[test]
fn project_flags_a_missing_authored_source() {
    let dir = Dir::new();
    let present = dir.file("present.step", b"s");
    let absent = dir.0.join("absent.step");
    let cfg = manifest(&[("present", &present), ("absent", &absent)]);
    let state = super::state::State::new(&[]);
    let text = super::project::project_json(&state, Some(&cfg), &dir.0);
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    let exists = |name: &str| {
        let bodies = json["bodies"].as_array().unwrap();
        bodies.iter().find(|b| b["name"] == name).unwrap()["exists"].clone()
    };
    assert_eq!(exists("present"), true);
    assert_eq!(exists("absent"), false);
    assert!(exists("lid").is_null());
}
