use std::path::Path;

use super::pipeline::{execute_pipeline, revision};
use super::server::route_path;
use super::watcher::is_relevant_change;

#[test]
fn pipeline_runs_on_valid_circuit_json() {
    let (payload, source) = execute_pipeline(Path::new("examples/circuit.json"));
    assert!(!payload.fuse_blown);
    assert!(payload.diagnostics.is_empty());
    assert!(payload.circuit_json.is_some());
    assert_eq!(payload.revision, revision(&source.unwrap()));
}

#[test]
fn pipeline_flags_blown_fuse_on_floating_net() {
    let (payload, _) = execute_pipeline(Path::new("examples/circuit-floating.json"));
    assert!(payload.fuse_blown);
    assert!(!payload.diagnostics.is_empty());
    assert!(payload
        .diagnostics
        .iter()
        .any(|d| d.rule == "unconnected_net"));
}

#[test]
fn pipeline_handles_missing_file_gracefully() {
    let (payload, source) = execute_pipeline(Path::new("nonexistent.json"));
    assert!(payload.fuse_blown);
    assert_eq!(payload.diagnostics[0].rule, "source_error");
    assert!(payload.revision.is_empty());
    assert!(source.is_none());
}

#[test]
fn watcher_filter_ignores_only_paths_inside_watch_dir() {
    let root = Path::new("/home/me/.config/target/dist/board");
    assert!(is_relevant_change(root, &root.join("index.tsx")));
    assert!(is_relevant_change(root, &root.join("lib/part.ts")));
    assert!(!is_relevant_change(
        root,
        &root.join("node_modules/x/index.ts")
    ));
    assert!(!is_relevant_change(root, &root.join("dist/index.js")));
    assert!(!is_relevant_change(root, &root.join("target/x.ts")));
    assert!(!is_relevant_change(root, &root.join(".hidden/a.tsx")));
    assert!(!is_relevant_change(root, &root.join("circuit.json")));
    assert!(!is_relevant_change(root, &root.join("notes.md")));
}

#[test]
fn route_path_strips_query_strings() {
    assert_eq!(route_path("/api/mechanical?rev=abc"), "/api/mechanical");
    assert_eq!(
        route_path("/api/mechanical.glb?t=1&x=2"),
        "/api/mechanical.glb"
    );
    assert_eq!(route_path("/?"), "/");
    assert_eq!(route_path("/events"), "/events");
}
