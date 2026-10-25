use super::model::Settings;
use super::updates::{due, supported, CHECK_INTERVAL_SECS};

#[test]
fn only_appimage_builds_update_on_linux() {
    assert!(supported(true, true));
    assert!(!supported(true, false));
    assert!(supported(false, false));
}

#[test]
fn automatic_checks_run_once_a_day() {
    let now = 2_000_000_000;
    assert!(due(None, now));
    assert!(!due(Some(now - CHECK_INTERVAL_SECS + 1), now));
    assert!(due(Some(now - CHECK_INTERVAL_SECS), now));
    assert!(!due(Some(now + 10), now));
}

#[test]
fn old_settings_files_gain_the_update_defaults() {
    let old: Settings = serde_json::from_str(r#"{"port":3021,"theme":"dark"}"#).unwrap();
    assert!(old.auto_update);
    assert_eq!(old.last_update_check, None);
    assert_eq!(old.port, 3021);
}
