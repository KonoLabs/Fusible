use std::path::Path;

use super::hub::ClientHub;
use super::pipeline::execute_pipeline;
use super::server::is_allowed_host;
use super::watcher::is_relevant_change;

#[test]
fn pipeline_runs_on_valid_circuit_json() {
    let payload = execute_pipeline(Path::new("examples/circuit.json"));
    assert!(!payload.fuse_blown);
    assert!(payload.diagnostics.is_empty());
    assert!(!payload.pcb_svg.is_empty());
    assert!(!payload.schematic_svg.is_empty());
    assert!(payload.circuit_json.is_some());
    assert!(payload.pcb_svg.contains("<svg"));
    assert!(payload.schematic_svg.contains("<svg"));
}

#[test]
fn pipeline_flags_blown_fuse_on_floating_net() {
    let payload = execute_pipeline(Path::new("examples/circuit-floating.json"));
    assert!(payload.fuse_blown);
    assert!(!payload.diagnostics.is_empty());
    assert!(payload
        .diagnostics
        .iter()
        .any(|d| d.rule == "unconnected_net"));
}

#[test]
fn pipeline_handles_missing_file_gracefully() {
    let payload = execute_pipeline(Path::new("nonexistent.json"));
    assert!(payload.fuse_blown);
    assert_eq!(payload.diagnostics[0].rule, "source_error");
}

#[test]
fn client_hub_broadcast_and_pruning() {
    let hub = ClientHub::default();
    let (tx1, rx1) = std::sync::mpsc::channel();
    let (tx2, rx2) = std::sync::mpsc::channel();

    hub.add_client(tx1);
    hub.add_client(tx2);

    hub.broadcast(r#"{"test": true}"#);

    let msg1 = String::from_utf8(rx1.recv().unwrap()).unwrap();
    assert!(msg1.starts_with("data: {\"test\": true}\n\n"));

    let msg2 = String::from_utf8(rx2.recv().unwrap()).unwrap();
    assert!(msg2.starts_with("data: {\"test\": true}\n\n"));

    drop(rx2);
    hub.broadcast(r#"{"test": 2}"#);

    let msg3 = String::from_utf8(rx1.recv().unwrap()).unwrap();
    assert!(msg3.starts_with("data: {\"test\": 2}\n\n"));
}

#[test]
fn host_check_allows_only_loopback_on_port() {
    assert!(is_allowed_host(Some("localhost:3020"), 3020));
    assert!(is_allowed_host(Some("127.0.0.1:3020"), 3020));
    assert!(is_allowed_host(Some("[::1]:3020"), 3020));
    assert!(is_allowed_host(Some("LocalHost:3020"), 3020));
    assert!(!is_allowed_host(Some("localhost:3021"), 3020));
    assert!(!is_allowed_host(Some("localhost"), 3020));
    assert!(!is_allowed_host(Some("evil.example:3020"), 3020));
    assert!(!is_allowed_host(Some("localhost.evil.example:3020"), 3020));
    assert!(!is_allowed_host(None, 3020));
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

/// Returns the body of every `useEffect` callback in the studio source, matched by brace depth.
fn effect_bodies(source: &str) -> Vec<&str> {
    let marker = "useEffect(() => {";
    source
        .match_indices(marker)
        .filter_map(|(start, _)| {
            let body_start = start + marker.len();
            let mut depth = 1usize;
            source[body_start..].char_indices().find_map(|(i, c)| {
                match c {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                (depth == 0).then(|| &source[body_start..body_start + i])
            })
        })
        .collect()
}

/// Reports whether `body` calls a React hook, i.e. a bare `useXxx(` identifier.
fn calls_hook(body: &str) -> bool {
    body.match_indices("use").any(|(i, _)| {
        let standalone = body[..i]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '.');
        let rest = &body[i + 3..];
        let ident_len = rest
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(rest.len());
        standalone
            && rest.starts_with(|c: char| c.is_ascii_uppercase())
            && rest[ident_len..].starts_with('(')
    })
}

#[test]
fn studio_effects_do_not_call_hooks() {
    assert!(calls_hook(
        "\n    useContextMenuIntercept(circuitJson, cb);\n"
    ));
    assert!(!calls_hook("container.useThing(); const user = 1;"));
    let bodies = effect_bodies(include_str!("../../studio/src/index.ts"));
    assert!(bodies.len() >= 3);
    for body in bodies {
        assert!(!calls_hook(body), "hook called inside useEffect:\n{body}");
    }
}
