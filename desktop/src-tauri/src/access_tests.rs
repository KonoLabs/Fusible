use super::access::{check_fragment, studio_pattern};
use tauri::Url;

#[test]
fn only_known_fragments_pass() {
    assert_eq!(check_fragment(None), Ok(None));
    for name in ["new", "open", "settings"] {
        assert_eq!(check_fragment(Some(name)), Ok(Some(name)));
    }
    assert!(check_fragment(Some("evil")).is_err());
    assert!(check_fragment(Some("")).is_err());
}

#[test]
fn studio_pattern_is_scoped_to_the_origin() {
    let url = Url::parse("http://localhost:4173/project").unwrap();
    assert_eq!(studio_pattern(&url), "http://localhost:4173/*");
}
