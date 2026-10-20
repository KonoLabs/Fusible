use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::commands::init;
use crate::commands::target::{is_tsx_target, resolve_entry};

struct TestDir {
    pub path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let path = std::env::temp_dir().join(format!("fusible_cmd_test_{pid}_{id}_{name}"));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("failed to create test dir");
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[test]
fn detects_tsx_target_extensions() {
    assert!(is_tsx_target(Path::new("board.tsx")));
    assert!(is_tsx_target(Path::new("circuit.ts")));
    assert!(is_tsx_target(Path::new("src/main.jsx")));
    assert!(is_tsx_target(Path::new("index.js")));
    assert!(!is_tsx_target(Path::new("circuit.json")));
    assert!(!is_tsx_target(Path::new("README.md")));
}

#[test]
fn resolves_explicit_entry_first() {
    let explicit = Some(PathBuf::from("custom/path.json"));
    let res = resolve_entry(explicit).expect("must resolve explicit");
    assert_eq!(res, PathBuf::from("custom/path.json"));
}

#[test]
fn init_creates_manifest_in_temp_dir() {
    let fixture = TestDir::new("init");
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&fixture.path).unwrap();

    let res = init::run(
        None,
        Some("custom-device".into()),
        Some("esp32-s3".into()),
        true,
    );
    assert!(res.is_ok());
    assert!(fixture.path.join("fusible.yml").exists());
    assert!(fixture.path.join("hardware/index.tsx").exists());
    assert!(fixture.path.join("hardware/package.json").exists());

    let content = std::fs::read_to_string(fixture.path.join("fusible.yml")).unwrap();
    assert!(content.contains("name: custom-device"));
    assert!(content.contains("chip: esp32-s3"));

    // Second init should fail because file already exists
    let res2 = init::run(None, None, None, true);
    assert!(res2.is_err());

    std::env::set_current_dir(prev).unwrap();
}
