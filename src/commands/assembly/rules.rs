//! The cross-world fuse rules: they turn probe measurements into violations and `check.json`.
//!
//! The board side of each rule is a set of probe solids built from the Circuit JSON (see
//! `probe.rs`); the body side is whatever the bodies' models occupy. The script only measures
//! how much each probe and body share. Every threshold and every decision lives here.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::probe::{Probe, Role};
pub(super) use super::rules_hole::missing_bosses;
use super::rules_hole::{aligned_quadrants, hole_finding, Group};
use crate::linter::Diagnostic;

/// Any shared volume above this counts as contact, in mm³.
pub const EPS_VOLUME: f64 = 0.01;

/// Every probe shrinks by this on each face, so parts that only touch do not fire, in mm.
pub const INSET: f64 = 0.1;

/// Radial width of the ring that must land on a boss around a hole, in mm.
pub const RING_WIDTH: f64 = 0.8;

/// Height of the hole probes above and below the board faces, in mm.
pub const PROBE_DEPTH: f64 = 2.0;

/// How much narrower than the hole the bore probe is, so a pilot hole under the screw size stays legal, in mm.
pub const BORE_MARGIN: f64 = 0.5;

/// How far from the hole edge a body counts as a boss meant for that hole, in mm.
pub const NEAR_MARGIN: f64 = 4.0;

/// How far outward from the board edge a connector must stay clear, in mm.
pub const OPENING_DEPTH: f64 = 3.0;

/// How tall a connector without a 3D model height is assumed to be above its board face, in mm.
pub const UNMEASURED_HEIGHT: f64 = 2.0;

pub const BODY_INTERFERENCE: &str = "body_interference";
pub const HOLE_MISALIGNED: &str = "hole_misaligned";
pub const CONNECTOR_BLOCKED: &str = "connector_blocked";

/// How much one probe and one body share.
#[derive(Debug, Clone, Deserialize)]
pub struct Measurement {
    pub probe: String,
    pub body: String,
    pub volume: f64,
    #[serde(default)]
    pub at: Option<[f64; 3]>,
    #[serde(default)]
    pub collision: Option<String>,
}

/// How much two bodies share.
#[derive(Debug, Clone, Deserialize)]
pub struct PairMeasurement {
    pub a: String,
    pub b: String,
    pub volume: f64,
    #[serde(default)]
    pub at: Option<[f64; 3]>,
    #[serde(default)]
    pub collision: Option<String>,
}

/// What the measure script writes: every non-empty overlap and the bodies it could not use.
#[derive(Debug, Default, Deserialize)]
pub struct Response {
    #[serde(default)]
    pub hits: Vec<Measurement>,
    #[serde(default)]
    pub pairs: Vec<PairMeasurement>,
    #[serde(default)]
    pub invalid: Vec<String>,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub measured: Vec<String>,
}

/// One broken rule, as written to `check.json`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Violation {
    pub rule: &'static str,
    pub message: String,
    pub parts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collision: Option<String>,
}

impl Violation {
    fn from_hit(rule: &'static str, message: String, parts: [&str; 2], hit: &Measurement) -> Self {
        Self {
            rule,
            message,
            parts: parts.map(str::to_string).to_vec(),
            volume: Some(hit.volume),
            at: hit.at,
            collision: hit.collision.clone(),
        }
    }

    /// The linter diagnostic that blows the fuse for this violation.
    pub fn diagnostic(&self) -> Diagnostic {
        let help = match self.rule {
            BODY_INTERFERENCE => {
                "move the body or the part, or reshape the body around the board; two bodies \
                 that overlap by more than 0.01 mm³ blow the fuse, so give a lip a clearance"
            }
            HOLE_MISALIGNED => {
                "line the boss up with the hole and keep its bore at least as wide, or remove the boss"
            }
            _ => "open the wall in front of the connector or move the body away from the edge",
        };
        Diagnostic {
            rule: self.rule.to_string(),
            message: self.message.clone(),
            help: Some(help.to_string()),
            ..Default::default()
        }
    }
}

pub(super) fn solid(volume: f64) -> bool {
    volume > EPS_VOLUME
}

fn violation_of(
    probe: &Probe,
    hit: &Measurement,
    quadrants: &HashMap<Group, usize>,
) -> Option<Violation> {
    let (body, board, volume) = (hit.body.as_str(), probe.board.as_str(), hit.volume);
    let (rule, message) = match &probe.role {
        Role::Slab => (
            BODY_INTERFERENCE,
            format!("body `{body}` overlaps board `{board}` by {volume:.2} mm³"),
        ),
        Role::KeepOut { component } => (
            BODY_INTERFERENCE,
            format!(
                "body `{body}` overlaps component `{component}` of board `{board}` by {volume:.2} mm³"
            ),
        ),
        Role::Opening { component, side } => (
            CONNECTOR_BLOCKED,
            format!(
                "body `{body}` blocks the opening of `{component}` on the {side} side of board `{board}` \
                 by {volume:.2} mm³"
            ),
        ),
        Role::HoleNear { .. } | Role::HoleBore { .. } => hole_finding(probe, hit, quadrants)?,
        Role::HoleQuadrant { .. } => return None,
    };
    Some(Violation::from_hit(rule, message, [body, board], hit))
}

/// Turns the measurements of `probes` into violations, in measurement order.
pub fn violations(probes: &[Probe], response: &Response) -> Vec<Violation> {
    let roles: HashMap<&str, &Probe> = probes.iter().map(|p| (p.id.as_str(), p)).collect();
    let quadrants = aligned_quadrants(&roles, &response.hits);
    let from_probes = response
        .hits
        .iter()
        .filter(|hit| solid(hit.volume))
        .filter_map(|hit| violation_of(roles.get(hit.probe.as_str())?, hit, &quadrants));
    let from_pairs = response
        .pairs
        .iter()
        .filter(|pair| solid(pair.volume))
        .map(|pair| Violation {
            rule: BODY_INTERFERENCE,
            message: format!(
                "body `{}` overlaps body `{}` by {:.2} mm³",
                pair.a, pair.b, pair.volume
            ),
            parts: vec![pair.a.clone(), pair.b.clone()],
            volume: Some(pair.volume),
            at: pair.at,
            collision: pair.collision.clone(),
        });
    from_probes.chain(from_pairs).collect()
}

#[derive(Serialize)]
struct Report<'a> {
    version: u32,
    fuse: &'static str,
    violations: &'a [Violation],
}

/// Writes `check.json` at `path`: `intact` without violations, `blown` with any.
pub fn write_report(path: &Path, violations: &[Violation]) -> std::io::Result<()> {
    let report = Report {
        version: 1,
        fuse: if violations.is_empty() {
            "intact"
        } else {
            "blown"
        },
        violations,
    };
    let text = serde_json::to_string_pretty(&report).map_err(std::io::Error::other)?;
    super::atomic::write_atomic(path, (text + "\n").as_bytes())
}
