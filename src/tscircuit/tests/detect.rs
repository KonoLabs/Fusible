//! Unit tests for detection precedence and path lookup.

use std::ffi::OsString;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};

use crate::tscircuit::detect::detect_runner;
use crate::tscircuit::error::RunnerError;
use crate::tscircuit::RunnerKind;

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(prefix: &str) -> Self {
        let unique = format!(
            "{prefix}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn create_executable(path: &Path) {
    if let Some(parent) = path.parent() {
        create_dir_all(parent).unwrap();
    }
    File::create(path).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms).unwrap();
    }
}

#[test]
fn detects_local_node_modules_tsci_first() {
    let temp = TempDirGuard::new("fusible_detect_local");
    let local_bin = temp.path.join("node_modules/.bin/tsci");
    create_executable(&local_bin);

    let global_dir = temp.path.join("global");
    create_executable(&global_dir.join("tsci"));

    let path_os = std::env::join_paths([&global_dir]).unwrap();
    let runner = detect_runner(&temp.path, Some(&path_os)).expect("runner detected");

    assert_eq!(runner.kind(), &RunnerKind::Local(local_bin));
}

#[test]
fn detects_global_tsci_when_local_absent() {
    let temp = TempDirGuard::new("fusible_detect_global");
    let global_dir = temp.path.join("global");
    let global_bin = global_dir.join("tsci");
    create_executable(&global_bin);

    let bun_dir = temp.path.join("bun_bin");
    create_executable(&bun_dir.join("bun"));

    let path_os = std::env::join_paths([&global_dir, &bun_dir]).unwrap();
    let runner = detect_runner(&temp.path, Some(&path_os)).expect("runner detected");

    assert_eq!(runner.kind(), &RunnerKind::Global(global_bin));
}

#[test]
fn detects_bun_before_npx() {
    let temp = TempDirGuard::new("fusible_detect_bun");
    let bun_dir = temp.path.join("bun_dir");
    let bun_bin = bun_dir.join("bun");
    create_executable(&bun_bin);

    let npx_dir = temp.path.join("npx_dir");
    create_executable(&npx_dir.join("npx"));

    let path_os = std::env::join_paths([&bun_dir, &npx_dir]).unwrap();
    let runner = detect_runner(&temp.path, Some(&path_os)).expect("runner detected");

    assert_eq!(runner.kind(), &RunnerKind::Bun(bun_bin));
}

#[test]
fn detects_npx_when_only_npx_present() {
    let temp = TempDirGuard::new("fusible_detect_npx");
    let npx_dir = temp.path.join("npx_dir");
    let npx_bin = npx_dir.join("npx");
    create_executable(&npx_bin);

    let path_os = std::env::join_paths([&npx_dir]).unwrap();
    let runner = detect_runner(&temp.path, Some(&path_os)).expect("runner detected");

    assert_eq!(runner.kind(), &RunnerKind::Npx(npx_bin));
}

#[test]
fn detection_fails_with_not_found_when_no_runner() {
    let temp = TempDirGuard::new("fusible_detect_empty");
    let empty_path = OsString::new();
    let err = detect_runner(&temp.path, Some(&empty_path)).expect_err("should fail");

    assert!(matches!(err, RunnerError::NotFound));
}
