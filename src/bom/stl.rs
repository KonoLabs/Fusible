//! Filament weight of a body, measured from the volume of its STL.

use std::path::Path;

const PLA_G_PER_CM3: f64 = 1.24;
const EFFECTIVE_FILL: f64 = 0.4;

fn volume_mm3(bytes: &[u8]) -> Option<f64> {
    let triangles = bytes
        .get(80..84)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize)?;
    let binary = bytes.len() == 84 + triangles * 50;
    let facets = if binary {
        binary_facets(bytes, triangles)
    } else {
        ascii_facets(std::str::from_utf8(bytes).ok()?)
    };
    let signed: f64 = facets.iter().map(signed_volume).sum();
    (!facets.is_empty()).then_some(signed.abs())
}

type Facet = [[f64; 3]; 3];

fn binary_facets(bytes: &[u8], count: usize) -> Vec<Facet> {
    let float = |at: usize| {
        f64::from(f32::from_le_bytes([
            bytes[at],
            bytes[at + 1],
            bytes[at + 2],
            bytes[at + 3],
        ]))
    };
    (0..count)
        .map(|i| {
            let base = 84 + i * 50 + 12;
            let vertex = |v: usize| {
                [
                    float(base + v * 12),
                    float(base + v * 12 + 4),
                    float(base + v * 12 + 8),
                ]
            };
            [vertex(0), vertex(1), vertex(2)]
        })
        .collect()
}

fn ascii_facets(text: &str) -> Vec<Facet> {
    let points: Vec<[f64; 3]> = text
        .lines()
        .filter_map(|line| line.trim().strip_prefix("vertex"))
        .filter_map(|rest| {
            let mut n = rest.split_whitespace().map(str::parse::<f64>);
            Some([n.next()?.ok()?, n.next()?.ok()?, n.next()?.ok()?])
        })
        .collect();
    points.as_chunks::<3>().0.to_vec()
}

fn signed_volume([a, b, c]: &Facet) -> f64 {
    let cross = [
        b[1] * c[2] - b[2] * c[1],
        b[2] * c[0] - b[0] * c[2],
        b[0] * c[1] - b[1] * c[0],
    ];
    (a[0] * cross[0] + a[1] * cross[1] + a[2] * cross[2]) / 6.0
}

/// Filament weight of the STL at `path`, from its volume with a typical PLA shell and infill.
pub fn stl_grams(path: &Path) -> Option<f64> {
    let bytes = std::fs::read(path).ok()?;
    let cm3 = volume_mm3(&bytes)? / 1000.0;
    Some((cm3 * PLA_G_PER_CM3 * EFFECTIVE_FILL * 10.0).round() / 10.0)
}
