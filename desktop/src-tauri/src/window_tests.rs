use super::window::title_for;

#[test]
fn titles_follow_the_page() {
    assert_eq!(title_for(None), "Fusible Studio");
    assert_eq!(title_for(Some("FieldCam")), "FieldCam - Fusible Studio");
}
