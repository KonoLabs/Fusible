//! Cross-section checks: every reference between boards, firmware, and bodies must resolve.

use std::collections::BTreeSet;

use super::types::BodyConfig;
use super::FusibleConfig;

/// Generators a body can name in `generate:`.
pub const GENERATORS: &[&str] = &["enclosure"];

/// Fails on duplicate names, dangling board references, or ambiguous bodies.
pub(super) fn validate(cfg: &FusibleConfig) -> Result<(), String> {
    unique_names(cfg)?;
    for (name, fw) in &cfg.firmware {
        board_reference(cfg, "firmware", name, fw.board.as_deref())?;
    }
    for (name, body) in &cfg.bodies {
        body_kind(name, body)?;
        if body.generate.is_some() {
            board_reference(cfg, "body", name, body.around.as_deref())?;
        }
    }
    Ok(())
}

fn unique_names(cfg: &FusibleConfig) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    let names = cfg
        .boards
        .keys()
        .chain(cfg.firmware.keys())
        .chain(cfg.bodies.keys());
    for name in names {
        if !seen.insert(name) {
            return Err(format!(
                "the name `{name}` is used twice; names are unique across boards, firmware, and bodies"
            ));
        }
    }
    Ok(())
}

fn board_reference(
    cfg: &FusibleConfig,
    kind: &str,
    name: &str,
    board: Option<&str>,
) -> Result<(), String> {
    match board {
        Some(board) if !cfg.boards.contains_key(board) => {
            Err(format!("{kind} `{name}` refers to unknown board `{board}`"))
        }
        None if cfg.boards.len() > 1 => Err(format!(
            "{kind} `{name}` must name its board; the project has {} boards",
            cfg.boards.len()
        )),
        _ => Ok(()),
    }
}

fn body_kind(name: &str, body: &BodyConfig) -> Result<(), String> {
    match (&body.source, &body.generate) {
        (Some(_), Some(_)) => Err(format!(
            "body `{name}` sets both `source` and `generate`; a body is either authored or generated"
        )),
        (None, None) => Err(format!("body `{name}` needs a `source` file or a `generate` generator")),
        (None, Some(_)) if body.build.is_some() => {
            Err(format!("body `{name}` sets `build` without a `source` file for it to produce"))
        }
        (None, Some(generator)) if !GENERATORS.contains(&generator.as_str()) => Err(format!(
            "body `{name}` uses unknown generator `{generator}`; known generators: {}",
            GENERATORS.join(", ")
        )),
        _ => Ok(()),
    }
}
