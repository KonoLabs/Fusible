use super::actions::next_zoom;

#[test]
fn zoom_steps_and_clamps() {
    assert_eq!(next_zoom(1.0, "zoom-in"), 1.1);
    assert_eq!(next_zoom(1.0, "zoom-out"), 0.9);
    assert_eq!(next_zoom(2.0, "zoom-reset"), 1.0);
    assert_eq!(next_zoom(3.0, "zoom-in"), 3.0);
    assert_eq!(next_zoom(0.5, "zoom-out"), 0.5);
}
