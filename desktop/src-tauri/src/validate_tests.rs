use super::model::Settings;
use super::validate::{project_name, settings};

#[test]
fn names_are_trimmed_and_folder_safe() {
    assert_eq!(project_name("  FieldCam "), Ok("FieldCam".into()));
    for bad in ["", "  ", "..", "a/b", "a:b", "x.", "tab\tname"] {
        assert!(project_name(bad).is_err(), "{bad:?} should fail");
    }
}

#[test]
fn settings_are_normalized() {
    let input = Settings {
        projects_dir: " /p ".into(),
        default_currency: "usd".into(),
        ..Settings::default()
    };
    let clean = settings(input).unwrap();
    assert_eq!(clean.default_currency, "USD");
    assert_eq!(clean.projects_dir, "/p");
}

#[test]
fn bad_settings_are_refused() {
    let base = Settings {
        projects_dir: "/p".into(),
        ..Settings::default()
    };
    let port = Settings {
        port: 80,
        ..base.clone()
    };
    let theme = Settings {
        theme: "neon".into(),
        ..base.clone()
    };
    let currency = Settings {
        default_currency: "EURO".into(),
        ..base.clone()
    };
    let dir = Settings {
        projects_dir: " ".into(),
        ..base
    };
    for bad in [port, theme, currency, dir] {
        assert!(settings(bad).is_err());
    }
}

#[test]
fn update_preferences_survive_validation() {
    let input = Settings {
        projects_dir: "/p".into(),
        auto_update: false,
        last_update_check: Some(1_792_800_000),
        ..Settings::default()
    };
    let clean = settings(input).unwrap();
    assert!(!clean.auto_update);
    assert_eq!(clean.last_update_check, Some(1_792_800_000));
}
