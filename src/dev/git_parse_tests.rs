use super::git_error::human;
use super::git_porcelain::parse as parse_status;
use super::git_resolve::Keep;
use super::git_route::{is_project_file, parse, Route};

#[test]
fn porcelain_entries_become_plain_changes() {
    let raw = concat!(
        "# branch.oid abc\0# branch.head main\0# branch.upstream origin/main\0# branch.ab +2 -3\0",
        "1 .M N... 100644 100644 100644 a b my board.kicad_pcb\0",
        "1 A. N... 000000 100644 100644 a b new.txt\0",
        "1 .D N... 100644 100644 000000 a b gone.txt\0",
        "2 R. N... 100644 100644 100644 a b R100 renamed.txt\0old.txt\0",
        "u UU N... 100644 100644 100644 100644 a b c clash.kicad_pcb\0",
        "? loose file.txt\0",
    );
    let out = parse_status(raw);
    assert_eq!((out.ahead, out.behind), (2, 3));
    assert_eq!(out.branch.as_deref(), Some("main"));
    assert_eq!(out.upstream.as_deref(), Some("origin/main"));
    let seen: Vec<_> = out
        .changes
        .iter()
        .map(|c| (c.path.as_str(), c.kind))
        .collect();
    assert_eq!(
        seen,
        vec![
            ("my board.kicad_pcb", "modified"),
            ("new.txt", "added"),
            ("gone.txt", "deleted"),
            ("renamed.txt", "renamed"),
            ("clash.kicad_pcb", "conflict"),
            ("loose file.txt", "added"),
        ]
    );
}

#[test]
fn resolve_requests_are_validated() {
    let good = parse(
        "/api/git/resolve",
        r#"{"path":"a.kicad_pcb","keep":"mine"}"#,
    );
    assert_eq!(
        good,
        Ok(Route::Resolve {
            path: "a.kicad_pcb".into(),
            keep: Keep::Mine
        })
    );
    assert!(parse("/api/git/resolve", r#"{"path":"a","keep":"both"}"#).is_err());
    assert!(parse("/api/git/resolve", "{}").is_err());
    assert!(parse("/api/git/save", "nope").is_err());
    assert_eq!(parse("/api/git/update", ""), Ok(Route::Update));
}

#[test]
fn project_files_trigger_a_rebuild() {
    for path in [
        "a.kicad_pcb",
        "x/b.kicad_sch",
        "hardware/part.step",
        "main.tsx",
    ] {
        assert!(is_project_file(path), "{path}");
    }
    assert!(!is_project_file("notes.txt"));
}

#[test]
fn errors_are_put_in_plain_words() {
    assert!(human("fatal: could not read Username for 'https://x'").contains("sign-in"));
    assert!(human("fatal: Could not resolve host: x").contains("internet"));
    assert!(human("! [rejected] main -> main (fetch first)").contains("Get updates"));
    assert!(!human("boom").contains("git"));
}
