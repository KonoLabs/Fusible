//! STL solids built from triangles: boxes and square tubes.

#![allow(dead_code)]
pub type Point = [f64; 3];
pub type Triangle = [Point; 3];

const SQUARE: [[f64; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];

fn corner(scale: f64, i: usize, z: f64) -> Point {
    [SQUARE[i][0] * scale, SQUARE[i][1] * scale, z]
}

pub fn reversed(tris: Vec<Triangle>) -> Vec<Triangle> {
    tris.into_iter().map(|[a, b, c]| [a, c, b]).collect()
}

pub fn walls(scale: f64, z0: f64, z1: f64) -> Vec<Triangle> {
    (0..4)
        .flat_map(|i| {
            let j = (i + 1) % 4;
            [
                [
                    corner(scale, i, z0),
                    corner(scale, j, z0),
                    corner(scale, j, z1),
                ],
                [
                    corner(scale, i, z0),
                    corner(scale, j, z1),
                    corner(scale, i, z1),
                ],
            ]
        })
        .collect()
}

pub fn cap(outer: f64, inner: f64, z: f64) -> Vec<Triangle> {
    (0..4)
        .flat_map(|i| {
            let j = (i + 1) % 4;
            [
                [
                    corner(outer, i, z),
                    corner(outer, j, z),
                    corner(inner, j, z),
                ],
                [
                    corner(outer, i, z),
                    corner(inner, j, z),
                    corner(inner, i, z),
                ],
            ]
        })
        .collect()
}

pub fn disc(scale: f64, z: f64) -> Vec<Triangle> {
    let c = |i| corner(scale, i, z);
    vec![[c(0), c(1), c(2)], [c(0), c(2), c(3)]]
}

pub fn box_triangles(half: f64, z0: f64, z1: f64) -> Vec<Triangle> {
    let mut tris = walls(half, z0, z1);
    tris.extend(disc(half, z1));
    tris.extend(reversed(disc(half, z0)));
    tris
}

pub fn tube_triangles(half: f64, bore: f64, z0: f64, z1: f64) -> Vec<Triangle> {
    let mut tris = walls(half, z0, z1);
    tris.extend(reversed(walls(bore, z0, z1)));
    tris.extend(cap(half, bore, z1));
    tris.extend(reversed(cap(half, bore, z0)));
    tris
}

pub fn stl(tris: &[Triangle]) -> String {
    let facets: String = tris
        .iter()
        .map(|tri| {
            let vertices: String = tri
                .iter()
                .map(|[x, y, z]| format!("      vertex {x} {y} {z}\n"))
                .collect();
            format!("  facet normal 0 0 0\n    outer loop\n{vertices}    endloop\n  endfacet\n")
        })
        .collect();
    format!("solid body\n{facets}endsolid body\n")
}
