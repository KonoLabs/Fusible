//! Cross-section checks: every reference between boards, firmware, and bodies must resolve.

use std::collections::BTreeSet;

use super::types::{AssemblyConfig, BodyConfig, LinkConfig};
use super::FusibleConfig;

/// Generators a body can name in `generate:`.
pub const GENERATORS: &[&str] = &["enclosure"];

/// Body names that `fusible body` reads as subcommands.
const RESERVED_BODY_NAMES: &[&str] = &["eject", "kit"];

/// Fails on duplicate names, dangling board references, or ambiguous bodies.
pub(super) fn validate(cfg: &FusibleConfig) -> Result<(), String> {
    unique_names(cfg)?;
    for (name, fw) in &cfg.firmware {
        board_reference(cfg, "firmware", name, fw.board.as_deref())?;
    }
    for (name, body) in &cfg.bodies {
        reserved_name(name)?;
        body_kind(name, body)?;
        if body.generate.is_some() || body.board.is_some() {
            board_reference(cfg, "body", name, body.board.as_deref())?;
        }
    }
    for (name, link) in &cfg.links {
        link_ends(cfg, name, link)?;
    }
    if let Some(assembly) = &cfg.assembly {
        assembly_parts(cfg, assembly)?;
    }
    Ok(())
}

fn assembly_parts(cfg: &FusibleConfig, assembly: &AssemblyConfig) -> Result<(), String> {
    if assembly.parts.is_empty() {
        return Err("`assembly` has no parts; list the boards and bodies to place".into());
    }
    for (name, placement) in &assembly.parts {
        if cfg.boards.contains_key(name) {
            continue;
        }
        let Some(body) = cfg.bodies.get(name) else {
            return Err(format!(
                "assembly part `{name}` is not a board or a body of this project"
            ));
        };
        match cfg.body_board(body) {
            _ if placement.at.is_some() => {}
            None => {
                return Err(format!(
                    "assembly part `{name}` is a body without a board; give it an `at` position or a `board:`"
                ))
            }
            Some(board) if !assembly.parts.contains_key(board) => {
                return Err(format!(
                    "assembly part `{name}` sits on board `{board}`, which is not in the assembly; \
                     add `{board}` to the parts or give `{name}` an `at` position"
                ))
            }
            Some(_) => {}
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

fn reserved_name(name: &str) -> Result<(), String> {
    if RESERVED_BODY_NAMES.contains(&name) {
        return Err(format!(
            "body `{name}` uses a reserved name; `fusible body {name}` runs the `{name}` subcommand, \
             so pick another name"
        ));
    }
    Ok(())
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

fn link_ends(cfg: &FusibleConfig, name: &str, link: &LinkConfig) -> Result<(), String> {
    for end in [&link.from, &link.to] {
        let board = end
            .split_once('.')
            .filter(|(b, r)| !b.is_empty() && !r.is_empty());
        match board {
            None => {
                return Err(format!(
                    "link `{name}` end `{end}` must be written `board.REFDES`"
                ))
            }
            Some((board, _)) if !cfg.boards.contains_key(board) => {
                return Err(format!("link `{name}` refers to unknown board `{board}`"))
            }
            Some(_) => {}
        }
    }
    if link.wires.is_empty() {
        return Err(format!("link `{name}` has no wires"));
    }
    if link
        .wires
        .iter()
        .any(|w| w.net.is_some() && w.nets.is_some())
    {
        return Err(format!(
            "link `{name}` sets both `net` and `nets` on a wire; use one"
        ));
    }
    Ok(())
}
