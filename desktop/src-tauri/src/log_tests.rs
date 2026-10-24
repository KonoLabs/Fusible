use super::log::Log;

#[test]
fn keeps_lines_in_order() {
    let mut log = Log::default();
    log.push("one\r\n");
    log.push("two");
    assert_eq!(log.text(), "one\ntwo");
    log.clear();
    assert_eq!(log.text(), "");
}

#[test]
fn drops_the_oldest_text_past_the_cap() {
    let mut log = Log::default();
    log.push("first");
    for _ in 0..2000 {
        log.push(&"é".repeat(40));
    }
    assert!(log.text().len() <= 64 * 1024);
    assert!(!log.text().contains("first"));
}
