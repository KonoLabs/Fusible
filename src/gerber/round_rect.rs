//! Standard macro definition for rounded rectangle apertures.

use std::fmt::Write as FmtWrite;

/// Emits the macro and aperture definition of a rounded rectangle: two bars and four corner discs.
pub(super) fn write_round_rect(
    out: &mut String,
    id: u32,
    [width_nm, height_nm, radius_nm]: [i64; 3],
) {
    let mm = |nm: i64| nm as f64 / 1_000_000.0;
    let (w, h, r) = (mm(width_nm), mm(height_nm), mm(radius_nm));
    let (cx, cy) = (w / 2.0 - r, h / 2.0 - r);
    let _ = writeln!(out, "%AMRR{id}*");
    let _ = writeln!(out, "21,1,{w:.6},{:.6},0,0,0*", h - 2.0 * r);
    let _ = writeln!(out, "21,1,{:.6},{h:.6},0,0,0*", w - 2.0 * r);
    for (sx, sy) in [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
        let _ = writeln!(out, "1,1,{:.6},{:.6},{:.6}*", 2.0 * r, sx * cx, sy * cy);
    }
    let _ = writeln!(out, "%");
    let _ = writeln!(out, "%ADD{id}RR{id}*%");
}
