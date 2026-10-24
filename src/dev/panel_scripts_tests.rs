use super::assets::static_response;

#[test]
fn viewer_loads_the_firmware_and_drawer_scripts_the_server_serves() {
    let html = include_str!("../assets/viewer.html");
    for script in ["/firmware.js", "/drawer.js", "/bom.js", "/git.js"] {
        assert!(html.contains(&format!(r#"<script src="{script}"></script>"#)));
        let response = static_response(script, None).unwrap();
        assert_eq!(response.status_code().0, 200);
    }
}

#[test]
fn sync_panel_posts_json_and_stays_hidden_outside_a_work_tree() {
    let script = include_str!("../assets/git.js");
    assert!(script.contains("method === 'POST'"));
    assert!(script.contains("'Content-Type': 'application/json'"));
    assert!(script.contains("root.hidden = !shown"));
    let html = include_str!("../assets/viewer.html");
    assert!(html.contains(r#"<div id="sync-view" hidden></div>"#));
}
