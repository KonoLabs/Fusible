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
    let sources =
        studio_sources(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("studio/src"));
    let bodies: Vec<&str> = sources.iter().flat_map(|s| effect_bodies(s)).collect();
    assert!(bodies.len() >= 3);
    for body in bodies {
        assert!(!calls_hook(body), "hook called inside useEffect:\n{body}");
    }
}

/// Reads every TypeScript file under `dir`, recursing into subdirectories.
fn studio_sources(dir: &std::path::Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .expect("studio/src is readable")
        .flatten()
        .flat_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                studio_sources(&path)
            } else if path.extension().is_some_and(|e| e == "ts") {
                vec![std::fs::read_to_string(&path).expect("studio source is readable")]
            } else {
                Vec::new()
            }
        })
        .collect()
}
