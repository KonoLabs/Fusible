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
