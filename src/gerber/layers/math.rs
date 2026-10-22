//! Numeric validation and 2D geometry helpers shared by the layer builders.

use crate::gerber::coordinates::Coord;

/// Returns the value when present and finite, naming the field in the error otherwise.
pub(super) fn required(value: Option<f64>, name: &str) -> Result<f64, String> {
    match value {
        Some(v) if v.is_finite() => Ok(v),
        Some(v) => Err(format!("{name} is not finite ({v})")),
        None => Err(format!("missing {name}")),
    }
}

/// Returns `default` when absent, or the value when present and finite.
pub(super) fn finite_or(value: Option<f64>, default: f64, name: &str) -> Result<f64, String> {
    value.map_or(Ok(default), |v| required(Some(v), name))
}

/// Returns the rotation as a whole number of quarter turns, or `None` when it is not axis-aligned.
pub(super) fn quarter_turns(rot: f64) -> Option<u8> {
    let q = rot / 90.0;
    let r = q.round();
    ((q - r).abs() < 1e-9).then(|| r.rem_euclid(4.0) as u8)
}

/// Rotates a vector counter-clockwise by `deg` degrees.
pub(super) fn rotate([x, y]: [f64; 2], deg: f64) -> [f64; 2] {
    let (s, c) = deg.to_radians().sin_cos();
    [x * c - y * s, x * s + y * c]
}

/// Adds two vectors.
pub(super) fn add(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

/// Returns the endpoints of the centre line of a `w` x `h` obround rotated by `rot` at `at`.
pub(super) fn long_axis(w: f64, h: f64, rot: f64, at: [f64; 2]) -> ([f64; 2], [f64; 2]) {
    let half = (w.max(h) - w.min(h)) / 2.0;
    let axis = if w >= h { [half, 0.0] } else { [0.0, half] };
    let [dx, dy] = rotate(axis, rot);
    ([at[0] - dx, at[1] - dy], [at[0] + dx, at[1] + dy])
}

/// Converts a millimetre point to fixed-point coordinates.
pub(super) fn coord([x, y]: [f64; 2]) -> [Coord; 2] {
    [Coord::from_mm(x), Coord::from_mm(y)]
}
