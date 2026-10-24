use super::assets::static_response;

#[test]
fn viewer_loads_the_firmware_and_drawer_scripts_the_server_serves() {
    let html = include_str!("../assets/viewer.html");
    for script in ["/firmware.js", "/drawer.js", "/bom.js"] {
        assert!(html.contains(&format!(r#"<script src="{script}"></script>"#)));
        let response = static_response(script, None).unwrap();
        assert_eq!(response.status_code().0, 200);
    }
}
