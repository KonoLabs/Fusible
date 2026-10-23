use super::check::{CheckError, ReadError};
use super::check_merge::{label, merge};
use crate::linter::{Diagnostic, FuseBlown};

fn blown(message: &str) -> CheckError {
    let violation = Diagnostic {
        rule: "unconnected_net".into(),
        message: message.into(),
        ..Default::default()
    };
    CheckError::FuseBlown(FuseBlown::new(vec![violation]))
}

fn tool(path: &str) -> CheckError {
    CheckError::Read(ReadError {
        path: path.into(),
        source: std::io::Error::other("boom"),
    })
}

fn messages(error: Option<CheckError>) -> Vec<String> {
    match error {
        Some(CheckError::FuseBlown(b)) => b.violations.into_iter().map(|v| v.message).collect(),
        other => panic!("expected a blown fuse, got {other:?}"),
    }
}

#[test]
fn violations_of_several_boards_merge_with_board_prefixes() {
    let mut reported = Vec::new();
    let a = label(blown("net A floats"), Some("main"));
    let b = label(blown("net B floats"), Some("sensor"));
    let merged = merge(Some(a), Some(b), &mut |blown| reported.push(blown));
    assert_eq!(
        messages(merged),
        ["[main] net A floats", "[sensor] net B floats"]
    );
    assert!(reported.is_empty());
}

#[test]
fn single_board_messages_stay_unprefixed() {
    let single = label(blown("net A floats"), None);
    let merged = merge(None, Some(single), &mut |_| {});
    assert_eq!(messages(merged), ["net A floats"]);
}

#[test]
fn a_tool_error_wins_but_the_violation_report_is_kept() {
    for (first, next) in [(blown("v"), tool("b.json")), (tool("b.json"), blown("v"))] {
        let mut reported = Vec::new();
        let merged = merge(Some(first), Some(next), &mut |blown| reported.push(blown));
        assert!(matches!(merged, Some(CheckError::Read(_))));
        assert_eq!(reported.len(), 1);
        assert_eq!(reported[0].violations[0].message, "v");
    }
}

#[test]
fn the_first_tool_error_is_kept_and_nothing_is_a_success() {
    let mut reported = Vec::new();
    let merged = merge(Some(tool("a")), Some(tool("b")), &mut |b| reported.push(b));
    assert!(matches!(merged, Some(CheckError::Read(r)) if r.path == "a"));
    assert!(merge(None, None, &mut |_| {}).is_none());
    assert!(reported.is_empty());
}
