//! Unit tests for bun lookup and the bun runner fallback.

use super::detect::{create_executable, shim, TempDirGuard};
use crate::tscircuit::bun::resolve_bun;
use crate::tscircuit::detect::detect_runner;
use crate::tscircuit::RunnerKind;

#[test]
fn falls_back_to_bun_x_when_no_tsci_is_installed() {
    let temp = TempDirGuard::new("fusible_detect_bun");
    let bin_dir = temp.path.join("bin");
    let bun = bin_dir.join(shim("bun"));
    create_executable(&bun);

    let path_os = std::env::join_paths([&bin_dir]).unwrap();
    let runner = detect_runner(&temp.path, Some(&path_os)).expect("runner detected");

    assert_eq!(runner.kind(), &RunnerKind::Bun(bun));
}

#[test]
fn tsci_wins_over_bun() {
    let temp = TempDirGuard::new("fusible_detect_tsci_over_bun");
    let bin_dir = temp.path.join("bin");
    create_executable(&bin_dir.join(shim("bun")));
    create_executable(&bin_dir.join(shim("tsci")));

    let path_os = std::env::join_paths([&bin_dir]).unwrap();
    let runner = detect_runner(&temp.path, Some(&path_os)).expect("runner detected");

    assert!(matches!(runner.kind(), RunnerKind::Global(_)));
}

#[test]
fn fusible_bun_beats_the_bun_on_path() {
    let temp = TempDirGuard::new("fusible_bun_env");
    let bundled = temp.path.join("bundled").join(shim("bun"));
    let on_path = temp.path.join("bin").join(shim("bun"));
    create_executable(&bundled);
    create_executable(&on_path);

    let path_os = std::env::join_paths([temp.path.join("bin")]).unwrap();
    let found = resolve_bun(Some(bundled.as_os_str()), Some(&path_os));
    assert_eq!(found, Some(bundled));

    let missing = temp.path.join("gone").join(shim("bun"));
    let found = resolve_bun(Some(missing.as_os_str()), Some(&path_os));
    assert_eq!(found, Some(on_path));
}
