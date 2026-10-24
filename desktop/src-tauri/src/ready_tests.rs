use super::ready::{parse, Ready};

#[test]
fn parses_the_ready_line() {
    let line = "FUSIBLE_READY {\"url\":\"http://localhost:3020/\",\"port\":3020}\r\n";
    assert_eq!(
        parse(line),
        Some(Ready {
            url: "http://localhost:3020/".into(),
            port: 3020
        })
    );
}

#[test]
fn ignores_other_lines() {
    assert_eq!(parse("Studio on http://localhost:3020"), None);
    assert_eq!(parse("FUSIBLE_READY not-json"), None);
    assert_eq!(parse("FUSIBLE_READY {\"url\":\"x\"}"), None);
}
