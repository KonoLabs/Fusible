use super::*;

#[test]
fn stl_volume_gives_grams() {
    let dir = std::env::temp_dir().join(format!("fusible-stl-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("cube.stl");
    let mut text = String::from("solid c\n");
    let v = |x: f64, y: f64, z: f64| format!("vertex {x} {y} {z}\n");
    let faces = [
        [(0., 0., 0.), (0., 10., 0.), (10., 10., 0.)],
        [(0., 0., 0.), (10., 10., 0.), (10., 0., 0.)],
        [(0., 0., 10.), (10., 0., 10.), (10., 10., 10.)],
        [(0., 0., 10.), (10., 10., 10.), (0., 10., 10.)],
        [(0., 0., 0.), (10., 0., 0.), (10., 0., 10.)],
        [(0., 0., 0.), (10., 0., 10.), (0., 0., 10.)],
        [(0., 10., 0.), (0., 10., 10.), (10., 10., 10.)],
        [(0., 10., 0.), (10., 10., 10.), (10., 10., 0.)],
        [(0., 0., 0.), (0., 0., 10.), (0., 10., 10.)],
        [(0., 0., 0.), (0., 10., 10.), (0., 10., 0.)],
        [(10., 0., 0.), (10., 10., 0.), (10., 10., 10.)],
        [(10., 0., 0.), (10., 10., 10.), (10., 0., 10.)],
    ];
    for tri in faces {
        text.push_str("facet normal 0 0 0\nouter loop\n");
        for (x, y, z) in tri {
            text.push_str(&v(x, y, z));
        }
        text.push_str("endloop\nendfacet\n");
    }
    std::fs::write(&path, text + "endsolid c\n").unwrap();
    assert_eq!(stl_grams(&path), Some(0.5));
    std::fs::remove_dir_all(dir).unwrap();
}
