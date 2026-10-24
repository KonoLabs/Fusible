//! Selection of the bodies one `fusible body` run builds.

use std::path::PathBuf;

use crate::commands::target::{resolve_board, BoardTarget, TargetError};
use crate::config::{BodyConfig, FusibleConfig};

/// One body to build, with the board a generated body is fitted to.
pub(super) enum Job {
    Authored {
        name: String,
        body: BodyConfig,
    },
    Generated {
        name: String,
        body: BodyConfig,
        board: Box<BoardTarget>,
    },
}

/// Picks the bodies for `target`: a body name, a board, or every body of the manifest.
pub(super) fn select_jobs(
    cfg: &FusibleConfig,
    target: Option<PathBuf>,
) -> Result<Vec<Job>, TargetError> {
    let by_name = target
        .as_deref()
        .and_then(|t| t.to_str())
        .and_then(|n| cfg.bodies.get_key_value(n));
    if let Some((name, body)) = by_name {
        return Ok(vec![job(name, body)?]);
    }
    if target.is_none() && !cfg.bodies.is_empty() {
        return cfg
            .bodies
            .iter()
            .map(|(name, body)| job(name, body))
            .collect();
    }
    let board = resolve_board(target)?;
    let fitted: Vec<Job> = cfg
        .bodies
        .iter()
        .filter(|(_, body)| body.generate.is_some() && fits(body, &board))
        .map(|(name, body)| generated(name, body, board.clone()))
        .collect();
    if !fitted.is_empty() {
        return Ok(fitted);
    }
    let name = match (&board.name, cfg.boards.len()) {
        (Some(board), 2..) => format!("{board}-enclosure"),
        _ => "enclosure".to_string(),
    };
    Ok(vec![generated(&name, &BodyConfig::default(), board)])
}

fn job(name: &str, body: &BodyConfig) -> Result<Job, TargetError> {
    if body.source.is_some() {
        return Ok(Job::Authored {
            name: name.to_string(),
            body: body.clone(),
        });
    }
    let board = resolve_board(body.board.clone().map(PathBuf::from))?;
    Ok(generated(name, body, board))
}

fn generated(name: &str, body: &BodyConfig, board: BoardTarget) -> Job {
    Job::Generated {
        name: name.to_string(),
        body: body.clone(),
        board: Box::new(board),
    }
}

fn fits(body: &BodyConfig, board: &BoardTarget) -> bool {
    board.name.is_some() && (body.board.is_none() || body.board == board.name)
}
