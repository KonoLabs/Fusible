//! Helpers of the cross-world rule tests: a project with one STL box placed over a board.

use std::path::Path;

use crate::common::{fusible, TestDir};

const BOARD: &str = r#"[
  {"type":"pcb_board","width":40,"height":30,"center":{"x":0,"y":0},"thickness":1.6}
]"#;

const CORNERS: [[f64; 3]; 8] = [
    [-5.0, -5.0, -5.0],
    [5.0, -5.0, -5.0],
    [5.0, 5.0, -5.0],
    [-5.0, 5.0, -5.0],
    [-5.0, -5.0, 5.0],
    [5.0, -5.0, 5.0],
    [5.0, 5.0, 5.0],
    [-5.0, 5.0, 5.0],
];

const TRIANGLES: [[usize; 3]; 12] = [
    [0, 3, 2],
    [0, 2, 1],
    [4, 5, 6],
    [4, 6, 7],
    [0, 1, 5],
    [0, 5, 4],
    [2, 3, 7],
    [2, 7, 6],
    [0, 4, 7],
    [0, 7, 3],
    [1, 2, 6],
    [1, 6, 5],
];

pub fn bun_available() -> bool {
    if std::env::var_os("FUSIBLE_BUN").is_some() {
        return true;
    }
    let paths = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&paths).any(|dir| dir.join("bun").is_file())
}

fn stl_box() -> String {
    let facets: String = TRIANGLES
        .iter()
        .map(|tri| {
            let vertices: String = tri
                .iter()
                .map(|&i| {
                    let [x, y, z] = CORNERS[i];
                    format!("      vertex {x} {y} {z}\n")
                })
                .collect();
            format!("  facet normal 0 0 0\n    outer loop\n{vertices}    endloop\n  endfacet\n")
        })
        .collect();
    format!("solid box\n{facets}endsolid box\n")
}

pub fn project(name: &str, at: &str) -> TestDir {
    let dir = TestDir::new(name);
    std::fs::write(dir.path.join("circuit.json"), BOARD).unwrap();
    std::fs::write(dir.path.join("box.stl"), stl_box()).unwrap();
    let manifest = format!(
        "boards:\n  main: {{ entry: circuit.json }}\n\
         bodies:\n  box: {{ source: box.stl, board: main }}\n\
         assembly: {{ parts: {{ box: {{ at: {at} }} }} }}\n"
    );
    std::fs::write(dir.path.join("fusible.yml"), manifest).unwrap();
    dir
}

pub fn run(dir: &Path, args: &[&str]) -> (Option<i32>, String) {
    let out = fusible()
        .current_dir(dir)
        .args(args)
        .output()
        .expect("binary should run");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

pub fn report(dir: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(dir.join("dist/assembly/check.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}
