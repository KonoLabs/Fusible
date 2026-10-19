//! Unit tests for configuration discovery, loading, parsing, and defaults.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;
use crate::config::error::ConfigError;

/// RAII helper for creating and cleaning up isolated test directories.
struct TestDir {
    pub path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let path = std::env::temp_dir().join(format!("fusible_test_{pid}_{id}_{name}"));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("failed to create test dir");
        Self { path }
    }

    fn write(&self, relative: &str, content: &str) -> PathBuf {
        let full = self.path.join(relative);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).expect("failed to create parent dirs");
        }
        std::fs::write(&full, content).expect("failed to write test file");
        full
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[test]
fn default_manifest_and_empty_inputs_parse_cleanly() {
    let manifest = FusibleConfig::default_manifest();
    let config = FusibleConfig::from_str(manifest).expect("default manifest should parse cleanly");

    assert_eq!(config.name.as_deref(), Some("sensor-node"));

    let hw = config.hardware.expect("hardware config present");
    assert_eq!(hw.entry, Some(PathBuf::from("hardware/index.tsx")));
    assert_eq!(hw.out_dir, Some(PathBuf::from("dist")));
    assert_eq!(hw.port, Some(3020));
    assert_eq!(hw.circuit_json, None);

    let mcu = config.mcu.expect("mcu config present");
    assert_eq!(mcu.chip.as_deref(), Some("stm32g431"));

    let fw = config.firmware.expect("firmware config present");
    assert_eq!(fw.path, Some(PathBuf::from("firmware")));
    assert_eq!(fw.framework.as_deref(), Some("embassy"));
    assert_eq!(fw.pins_rs, Some(PathBuf::from("firmware/src/pins.rs")));
    assert_eq!(fw.build_cmd.as_deref(), Some("cargo build"));

    // Empty and comment inputs
    assert_eq!(
        FusibleConfig::from_str("").expect("empty input"),
        FusibleConfig::default()
    );
    assert_eq!(
        FusibleConfig::from_str("  \n\t ").expect("whitespace"),
        FusibleConfig::default()
    );
    assert_eq!(
        FusibleConfig::from_str("# comment\n").expect("comment"),
        FusibleConfig::default()
    );
}

#[test]
fn parses_kebab_case_aliases() {
    let yaml = r#"
name: aliased-node
hardware:
  out-dir: build/dist
  circuit-json: build/circuit.json
firmware:
  pins-rs: generated/pins.rs
  build-cmd: make firmware
"#;
    let config = FusibleConfig::from_str(yaml).expect("aliased fields should parse");
    assert_eq!(config.name.as_deref(), Some("aliased-node"));

    let hw = config.hardware.expect("hardware present");
    assert_eq!(hw.out_dir, Some(PathBuf::from("build/dist")));
    assert_eq!(hw.circuit_json, Some(PathBuf::from("build/circuit.json")));

    let fw = config.firmware.expect("firmware present");
    assert_eq!(fw.pins_rs, Some(PathBuf::from("generated/pins.rs")));
    assert_eq!(fw.build_cmd.as_deref(), Some("make firmware"));
}

#[test]
fn handles_syntax_and_io_errors() {
    let invalid_yaml = "name: [\nunclosed bracket";
    let err = FusibleConfig::from_str(invalid_yaml).expect_err("syntax error must fail");
    match err {
        ConfigError::Parse { message, span, .. } => {
            assert!(!message.is_empty());
            assert!(span.offset() <= invalid_yaml.len());
        }
        ConfigError::Io { .. } => panic!("expected ConfigError::Parse"),
    }

    let missing = Path::new("this/path/does/not/exist/fusible.yml");
    match FusibleConfig::from_file(missing).expect_err("missing file") {
        ConfigError::Io { path, .. } => assert_eq!(path, missing),
        ConfigError::Parse { .. } => panic!("expected ConfigError::Io"),
    }
}

#[test]
fn find_and_load_discovery_and_precedence() {
    let fixture = TestDir::new("discovery");
    fixture.write("fusible.yml", "name: yml-manifest\n");
    fixture.write("fusible.yaml", "name: yaml-manifest\n");
    fixture.write("crates/firmware/src/main.rs", "fn main() {}\n");

    // Ancestor lookup from nested directory
    let sub_dir = fixture.path.join("crates/firmware/src");
    let (found_path, found_cfg) = FusibleConfig::find_and_load(&sub_dir)
        .expect("lookup should succeed")
        .expect("manifest found");
    assert_eq!(found_path, fixture.path.join("fusible.yml"));
    assert_eq!(found_cfg.name.as_deref(), Some("yml-manifest"));

    // Precedence: .yml over .yaml
    let (p, c) = FusibleConfig::find_and_load(&fixture.path)
        .expect("lookup")
        .expect("found");
    assert_eq!(p, fixture.path.join("fusible.yml"));
    assert_eq!(c.name.as_deref(), Some("yml-manifest"));

    // Falls back to .yaml when .yml is missing
    std::fs::remove_file(fixture.path.join("fusible.yml")).unwrap();
    let (p, c) = FusibleConfig::find_and_load(&fixture.path)
        .expect("lookup")
        .expect("found");
    assert_eq!(p, fixture.path.join("fusible.yaml"));
    assert_eq!(c.name.as_deref(), Some("yaml-manifest"));
}

#[test]
fn find_and_load_missing_and_direct_file() {
    let fixture = TestDir::new("missing_and_file");
    let nested = fixture.path.join("empty/sub");
    std::fs::create_dir_all(&nested).unwrap();

    // Not found
    assert!(FusibleConfig::find_and_load(&nested).unwrap().is_none());

    // Direct file path
    let file = fixture.write("fusible.yml", "name: direct-manifest\n");
    let (p, c) = FusibleConfig::find_and_load(&file)
        .expect("lookup")
        .expect("found");
    assert_eq!(p, file);
    assert_eq!(c.name.as_deref(), Some("direct-manifest"));
}
