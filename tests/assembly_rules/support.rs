//! Helpers of the cross-world rule tests: a project with one STL solid placed over a board.

#![allow(dead_code)]
use std::path::Path;

use crate::common::{fusible, require_bun, TestDir};
use crate::solids::{box_triangles, stl, Triangle};

pub const BOARD: &str = r#"[
  {"type":"pcb_board","width":40,"height":30,"center":{"x":0,"y":0},"thickness":1.6}
]"#;

pub const HOLE_BOARD: &str = r#"[
  {"type":"pcb_board","width":40,"height":30,"center":{"x":0,"y":0},"thickness":1.6},
  {"type":"pcb_hole","x":0,"y":0,"hole_diameter":3.2}
]"#;

pub fn bun_available() -> bool {
    let found = match std::env::var_os("FUSIBLE_BUN") {
        Some(path) => Path::new(&path).is_file().then_some(path.into()),
        None => require_bun(),
    };
    if found.is_none() && std::env::var("FUSIBLE_REQUIRE_BUN").as_deref() == Ok("1") {
        panic!("FUSIBLE_REQUIRE_BUN=1 but no usable bun was found");
    }
    found.is_some()
}

pub fn project_with(name: &str, at: &str, board: &str, solid: &[Triangle]) -> TestDir {
    let dir = TestDir::new(name);
    std::fs::write(dir.path.join("circuit.json"), board).unwrap();
    std::fs::write(dir.path.join("box.stl"), stl(solid)).unwrap();
    let manifest = format!(
        "boards:\n  main: {{ entry: circuit.json }}\n\
         bodies:\n  box: {{ source: box.stl, board: main }}\n\
         assembly: {{ parts: {{ box: {{ at: {at} }} }} }}\n"
    );
    std::fs::write(dir.path.join("fusible.yml"), manifest).unwrap();
    dir
}

pub fn project(name: &str, at: &str) -> TestDir {
    project_with(name, at, BOARD, &box_triangles(5.0, -5.0, 5.0))
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
