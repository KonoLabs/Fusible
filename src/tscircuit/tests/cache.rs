//! Tests for the build cache: a fake `tsci` counts how often it really runs.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use super::detect::TempDirGuard;
use crate::tscircuit::{Runner, RunnerKind};

struct Project {
    temp: TempDirGuard,
    runner: Runner,
}

impl Project {
    fn new(name: &str) -> Self {
        let temp = TempDirGuard::new(name);
        let tsci = temp.path.join("fake-tsci");
        let counter = temp.path.join("runs");
        let script = format!(
            "#!/bin/sh\necho run >> '{}'\nmkdir -p dist\nprintf '[]' > dist/circuit.json\n\
             echo \"Circuit JSON written to $PWD/dist/circuit.json\"\n",
            counter.display()
        );
        fs::write(&tsci, script).unwrap();
        fs::set_permissions(&tsci, fs::Permissions::from_mode(0o755)).unwrap();
        let ws = temp.path.join("ws");
        fs::create_dir_all(&ws).unwrap();
        fs::write(ws.join("package.json"), "{}").unwrap();
        fs::write(ws.join("board.tsx"), "export default 1").unwrap();
        let runner = Runner::new(RunnerKind::Global(tsci));
        Self { temp, runner }
    }

    fn ws(&self) -> PathBuf {
        self.temp.path.join("ws")
    }

    fn build(&self) -> PathBuf {
        self.runner
            .build(&self.ws().join("board.tsx"), None)
            .unwrap()
    }

    fn runs(&self) -> usize {
        fs::read_to_string(self.temp.path.join("runs")).map_or(0, |text| text.lines().count())
    }
}

#[test]
fn unchanged_sources_build_once() {
    let project = Project::new("fusible_cache_hit");
    let cached = |path: PathBuf| path.components().any(|part| part.as_os_str() == ".fusible");
    assert!(!cached(project.build()));
    assert!(cached(project.build()));
    assert_eq!(project.runs(), 1);
}

#[test]
fn an_edited_source_rebuilds() {
    let project = Project::new("fusible_cache_edit");
    project.build();
    fs::write(project.ws().join("part.tsx"), "export const r = 1").unwrap();
    project.build();
    assert_eq!(project.runs(), 2);
}

#[test]
fn build_output_and_dependencies_do_not_invalidate() {
    let project = Project::new("fusible_cache_ignored");
    project.build();
    fs::create_dir_all(project.ws().join("node_modules/x")).unwrap();
    fs::write(project.ws().join("node_modules/x/index.js"), "1").unwrap();
    project.build();
    assert_eq!(project.runs(), 1);
}

#[test]
fn an_import_outside_the_workspace_is_never_cached() {
    let project = Project::new("fusible_cache_escape");
    fs::write(
        project.ws().join("board.tsx"),
        "import a from \"../shared/a\"",
    )
    .unwrap();
    project.build();
    project.build();
    assert_eq!(project.runs(), 2);
}
