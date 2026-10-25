use super::shell_path::{extract, merge};

#[test]
fn merge_keeps_current_first_and_drops_duplicates() {
    assert_eq!(
        merge("/usr/bin:/bin", "/opt/x:/usr/bin"),
        "/usr/bin:/bin:/opt/x"
    );
}

#[test]
fn extract_ignores_shell_banners() {
    let out = "hello\n__FUSIBLE_PATH_BEGIN__/a:/b__FUSIBLE_PATH_END__bye";
    assert_eq!(extract(out), Some("/a:/b"));
    assert_eq!(extract("nothing"), None);
}
