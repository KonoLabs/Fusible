//! The theme file is the only place a colour is defined.

use std::fs;
use std::path::{Path, PathBuf};

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if name != "node_modules" && name != "dist" {
                sources(&path, out);
            }
        } else if !name.ends_with(".bundle.js") && !name.ends_with(".otf") {
            out.push(path);
        }
    }
}

#[test]
fn old_green_is_gone_everywhere() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for dir in ["src", "studio/src", "desktop/ui"] {
        sources(&root.join(dir), &mut files);
    }
    for file in files {
        let text = fs::read_to_string(&file).unwrap_or_default().to_lowercase();
        for old in ["86a881", "134, 168, 129"] {
            assert!(!text.contains(old), "{} still has {old}", file.display());
        }
    }
}

#[test]
fn theme_defines_green_and_surface() {
    let css = include_str!("../src/assets/theme.css");
    assert!(css.contains("#88A888"));
    assert!(css.contains("#111111"));
}
