//! Which manifest items belong to a BOM.

use std::path::{Path, PathBuf};

use super::estimate::Body;
use super::stl::stl_grams;
use crate::config::{BodyConfig, FusibleConfig};

fn stl_of(name: &str, body: &BodyConfig, root: &Path) -> PathBuf {
    match &body.source {
        Some(source)
            if source
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("stl")) =>
        {
            root.join(source)
        }
        _ => {
            let dir = body.enclosure.out_dir.clone();
            root.join(dir.unwrap_or_else(|| PathBuf::from("dist/bodies")))
                .join(format!("{name}.stl"))
        }
    }
}

/// Bodies that belong to one of `boards`, or to no board in particular, with the weight
/// measured from their STL under `root` when the file exists.
pub fn bodies_of(cfg: &FusibleConfig, boards: &[&str], root: &Path) -> Vec<Body> {
    cfg.bodies
        .iter()
        .filter(|(_, body)| body.board.as_deref().is_none_or(|b| boards.contains(&b)))
        .map(|(name, body)| Body {
            name: name.clone(),
            grams: stl_grams(&stl_of(name, body, root)),
        })
        .collect()
}
