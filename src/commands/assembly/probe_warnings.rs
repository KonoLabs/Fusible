//! Warnings about the assumptions a connector's probes rest on.

use super::rules::UNMEASURED_HEIGHT;
use crate::commands::body::Kit;

/// The lines that tell which assumptions a connector's probes rest on.
pub fn warnings(board: &str, kit: &Kit) -> Vec<String> {
    let mut lines = Vec::new();
    for opening in &kit.openings {
        let name = &opening.component;
        let component = kit.components.iter().find(|c| &c.name == name);
        if component.is_none_or(|c| c.z.is_none() || c.height.is_none()) {
            lines.push(format!(
                "connector `{name}` of board `{board}` has no 3D height; its opening is assumed \
                 {UNMEASURED_HEIGHT} mm tall"
            ));
        }
        if !kit.keep_out.iter().any(|k| &k.component == name) {
            lines.push(format!(
                "connector `{name}` of board `{board}` has no keep-out volume; a body overlapping \
                 it is not detected"
            ));
        }
    }
    lines
}
