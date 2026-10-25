//! Integration test for the `studio` subcommand and its legacy aliases.

mod common;

use common::fusible;

#[test]
fn studio_has_dev_view_and_webview_as_aliases() {
    let help = |name: &str| {
        let out = fusible()
            .args([name, "--help"])
            .output()
            .expect("binary should run");
        assert_eq!(out.status.code(), Some(0), "{name}");
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    let studio = help("studio");
    assert!(studio.contains("--desktop"));
    for alias in ["dev", "view", "webview"] {
        assert_eq!(help(alias), studio, "{alias}");
    }
}
