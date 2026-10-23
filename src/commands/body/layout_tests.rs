use super::layout::{lattice, notches, vents};
use super::settings::Ventilation;
use super::spec::{bounds_of, open_ring, Notch};
use super::CLEARANCE;
use crate::gerber::layers::{PcbBoardElm, PcbComponentElm, PcbPoint};

fn rect(w: f64, h: f64) -> Vec<PcbPoint> {
    open_ring(
        PcbBoardElm {
            width: Some(w),
            height: Some(h),
            ..Default::default()
        }
        .outline_points(),
    )
}

#[test]
fn notches_only_edge_components() {
    let part = |x: f64, y: f64, w: f64, h: f64| PcbComponentElm {
        center: Some(PcbPoint { x, y }),
        width: Some(w),
        height: Some(h),
        ..Default::default()
    };
    let out = notches(
        &[part(36.0, 0.0, 9.0, 7.0), part(0.0, 0.0, 9.0, 7.0)],
        &rect(80.0, 60.0),
        2.0,
    );
    assert_eq!(
        out,
        vec![Notch {
            points: [[40.0, -4.0], [40.0, 4.0], [43.5, 4.0], [43.5, -4.0]]
        }]
    );

    let tri = [
        PcbPoint { x: 0.0, y: 0.0 },
        PcbPoint { x: 60.0, y: 0.0 },
        PcbPoint { x: 0.0, y: 60.0 },
    ];
    let slanted = notches(&[part(28.0, 28.0, 4.0, 4.0)], &tri, 2.0);
    assert_eq!(slanted.len(), 1);
    let reach = (CLEARANCE + 2.0 + 1.0) * std::f64::consts::SQRT_2;
    let sums = slanted[0].points.map(|[x, y]| x + y);
    for (sum, expected) in sums.iter().zip([60.0, 60.0, 60.0 + reach, 60.0 + reach]) {
        assert!((sum - expected).abs() < 1e-9, "{sums:?}");
    }
}

#[test]
fn vents_stay_inside_the_outline() {
    let tri = vec![
        PcbPoint { x: 0.0, y: 0.0 },
        PcbPoint { x: 60.0, y: 0.0 },
        PcbPoint { x: 0.0, y: 60.0 },
    ];
    let grid = vents(Ventilation::Grid, &tri, bounds_of(&tri));
    assert!(!grid.is_empty());
    assert!(grid.iter().all(|v| v.x + v.y + 1.5 * 2.0 <= 60.0));
    assert!(vents(Ventilation::None, &tri, bounds_of(&tri)).is_empty());
}

#[test]
fn lattice_centers_features_in_the_span() {
    assert_eq!(lattice(0.0, 15.0, 3.0, 6.0), vec![1.5, 7.5, 13.5]);
    assert!(lattice(0.0, 2.0, 3.0, 6.0).is_empty());
}
