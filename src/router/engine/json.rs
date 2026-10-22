//! Field readers for raw Circuit JSON elements.

use serde_json::Value;

/// Half width and half height of a pad's axis-aligned bounding box,
/// accounting for `ccw_rotation`.
pub(super) fn pad_half_extent(el: &Value) -> Option<[f64; 2]> {
    let (w, h) = match (num(el, "width"), num(el, "height")) {
        (Some(w), Some(h)) => (w, h),
        _ => {
            let d = num(el, "outer_diameter")
                .or_else(|| num(el, "radius").map(|r| r * 2.0))
                .map(|d| (d, d));
            let outer = num(el, "outer_width").zip(num(el, "outer_height"));
            let rect = num(el, "rect_pad_width").zip(num(el, "rect_pad_height"));
            d.or(outer).or(rect)?
        }
    };
    let rot = ["ccw_rotation", "rect_ccw_rotation"]
        .iter()
        .find_map(|k| num(el, k))
        .unwrap_or(0.0)
        .to_radians();
    let (s, c) = (rot.sin().abs(), rot.cos().abs());
    Some([(w * c + h * s) / 2.0, (w * s + h * c) / 2.0])
}

pub(super) fn type_of(el: &Value) -> &str {
    str_of(el, "type").unwrap_or("")
}

pub(super) fn str_of<'a>(el: &'a Value, key: &str) -> Option<&'a str> {
    el.get(key).and_then(Value::as_str)
}

pub(super) fn num(el: &Value, key: &str) -> Option<f64> {
    el.get(key)
        .and_then(Value::as_f64)
        .filter(|v| v.is_finite())
}
