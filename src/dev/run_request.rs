//! Validation of `POST /api/run`: only manifest names reach the command line.

use serde::Deserialize;

use crate::config::FusibleConfig;

#[derive(Deserialize)]
struct Request {
    action: String,
    #[serde(default)]
    target: Option<String>,
}

/// A validated run: the label shown in the studio and the arguments for `fusible`.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Spec {
    pub label: String,
    pub args: Vec<String>,
}

fn known_targets<'a>(cfg: &'a FusibleConfig, action: &str) -> Option<Vec<&'a String>> {
    let boards = cfg.boards.keys();
    match action {
        "check" => Some(boards.collect()),
        "build" => Some(boards.chain(cfg.firmware.keys()).collect()),
        "body" => Some(boards.chain(cfg.bodies.keys()).collect()),
        _ => None,
    }
}

/// Parses a request body and checks the action and target against the manifest.
pub(super) fn parse(body: &str, cfg: Option<&FusibleConfig>) -> Result<Spec, String> {
    let request: Request = serde_json::from_str(body).map_err(|e| format!("bad request: {e}"))?;
    let default = FusibleConfig::default();
    let targets = known_targets(cfg.unwrap_or(&default), &request.action).ok_or_else(|| {
        format!(
            "unknown action `{}`; use check, build, or body",
            request.action
        )
    })?;
    let mut args = vec![request.action.clone()];
    let mut label = request.action;
    if let Some(target) = request.target {
        if !targets.contains(&&target) {
            return Err(format!("`{target}` is not a valid target for `{label}`"));
        }
        label = format!("{label} {target}");
        args.push("--".into());
        args.push(target);
    }
    Ok(Spec { label, args })
}
