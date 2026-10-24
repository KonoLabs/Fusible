use super::assets::static_response;
use super::desktop::{ready_line, wait_for_eof};

fn page(desktop: bool) -> String {
    let response = static_response("/", None, desktop).unwrap();
    let mut body = String::new();
    std::io::Read::read_to_string(&mut response.into_reader(), &mut body).unwrap();
    body
}

#[test]
fn viewer_has_no_projects_button_without_the_desktop_flag() {
    let html = page(false);
    assert!(!html.contains("FUSIBLE_DESKTOP"));
    assert!(!html.contains("open_projects"));
}

#[test]
fn viewer_carries_the_flag_and_the_projects_button_under_the_desktop_shell() {
    let html = page(true);
    let flag = html.find("window.FUSIBLE_DESKTOP = true;").unwrap();
    assert!(flag < html.find("studio.bundle.js").unwrap());
    assert!(html.contains("__TAURI_INTERNALS__"));
    assert!(html.contains("open_projects"));
}

#[test]
fn ready_line_is_one_line_of_json_after_the_marker() {
    let line = ready_line(3021);
    let json = line.strip_prefix("FUSIBLE_READY ").unwrap();
    assert!(!line.contains('\n'));
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(value["port"], 3021);
    assert_eq!(value["url"], "http://localhost:3021/");
}

#[test]
fn wait_for_eof_returns_once_the_reader_is_drained() {
    wait_for_eof(&b"never read by anyone"[..]);
}
