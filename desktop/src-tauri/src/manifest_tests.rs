use super::manifest::{parse, Summary};

#[test]
fn counts_the_three_sections() {
    let text = "name: Demo\nboards:\n  a: {}\n  b: {}\nfirmware:\n  fw: {}\nbodies: {}\nextra: 1\n";
    let summary = parse(text);
    assert_eq!(
        summary,
        Summary {
            name: Some("Demo".into()),
            boards: 2,
            firmware: 1,
            bodies: 0,
        }
    );
}

#[test]
fn legacy_or_broken_manifest_counts_zero() {
    assert_eq!(parse("board: main.tsx\n"), Summary::default());
    assert_eq!(parse("boards: [unclosed"), Summary::default());
    assert_eq!(parse("boards: nope\n").boards, 0);
}

#[test]
fn blank_name_is_none() {
    assert_eq!(parse("name: '  '\n").name, None);
}
