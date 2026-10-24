//! Placement transforms of the assembly, as 3x4 row-major affine matrices.
//!
//! The layout is the one occt-wasm's `transform` takes:
//! `[r00, r01, r02, tx, r10, r11, r12, ty, r20, r21, r22, tz]`.

use crate::config::PlacementConfig;

/// A 3x4 row-major affine transform in millimetres.
pub type Matrix = [f64; 12];

/// The transform that leaves every point where it is.
pub const IDENTITY: Matrix = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];

/// Turns `placement` into a transform: rotate around fixed X, then Y, then Z, then translate.
pub fn placement(placement: &PlacementConfig) -> Matrix {
    let [x, y, z] = placement.rotate.unwrap_or_default().map(f64::to_radians);
    let [tx, ty, tz] = placement.at.unwrap_or_default();
    let (sx, cx) = x.sin_cos();
    let (sy, cy) = y.sin_cos();
    let (sz, cz) = z.sin_cos();
    [
        cz * cy,
        cz * sy * sx - sz * cx,
        cz * sy * cx + sz * sx,
        tx,
        sz * cy,
        sz * sy * sx + cz * cx,
        sz * sy * cx - cz * sx,
        ty,
        -sy,
        cy * sx,
        cy * cx,
        tz,
    ]
}

/// A pure translation by `[dx, dy, dz]`.
pub fn translation([dx, dy, dz]: [f64; 3]) -> Matrix {
    let mut m = IDENTITY;
    m[3] = dx;
    m[7] = dy;
    m[11] = dz;
    m
}

/// Returns `outer` after `inner`: a point moves by `inner` first, then by `outer`.
pub fn compose(outer: &Matrix, inner: &Matrix) -> Matrix {
    let mut m = [0.0; 12];
    for row in 0..3 {
        for col in 0..4 {
            let carried = (0..3)
                .map(|k| outer[row * 4 + k] * inner[k * 4 + col])
                .sum::<f64>();
            m[row * 4 + col] = carried + if col == 3 { outer[row * 4 + 3] } else { 0.0 };
        }
    }
    m
}

/// Applies `m` to the point `p`.
pub fn apply(m: &Matrix, p: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|row| (0..3).map(|k| m[row * 4 + k] * p[k]).sum::<f64>() + m[row * 4 + 3])
}
