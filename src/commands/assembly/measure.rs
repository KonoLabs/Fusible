//! The measure step: probes against bodies in one OpenCascade run, then the rules.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use super::parts::{Context, Part};
use super::place::IDENTITY;
use super::probe::{probes, Probe};
use super::rules::{violations, write_report, Response, Violation, EPS_VOLUME};
use super::AssemblyError;
use crate::commands::body::{build_kit, generator_dir, run_script};

static REPORT_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
struct Job<'a> {
    mode: &'static str,
    parts: &'a [Part],
    probes: &'a [Probe],
    pairs: Vec<[&'a str; 2]>,
    min_volume: f64,
    report: &'a Path,
    preview: Option<&'a Path>,
}

fn all_probes(ctx: &Context) -> Result<Vec<Probe>, AssemblyError> {
    let mut all = Vec::new();
    for (name, (_, json)) in ctx.circuits {
        let kit = build_kit(json, Some(name.clone()))?;
        let frame = ctx.frames.get(name).unwrap_or(&IDENTITY);
        all.extend(probes(name, &kit, frame));
    }
    Ok(all)
}

fn body_pairs(bodies: &[Part]) -> Vec<[&str; 2]> {
    let names: Vec<&str> = bodies.iter().map(|b| b.name.as_str()).collect();
    names
        .iter()
        .enumerate()
        .flat_map(|(i, a)| names[i + 1..].iter().map(move |b| [*a, *b]))
        .collect()
}

fn run_measure(
    bodies: &[Part],
    probes: &[Probe],
    preview: Option<&Path>,
) -> Result<Response, AssemblyError> {
    println!("📐 Measuring body clearances with OpenCascade...");
    let id = REPORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let report = generator_dir().join(format!("measure-{}-{id}.json", std::process::id()));
    let job = Job {
        mode: "measure",
        parts: bodies,
        probes,
        pairs: body_pairs(bodies),
        min_volume: EPS_VOLUME,
        report: &report,
        preview,
    };
    let payload = serde_json::to_vec(&job).map_err(std::io::Error::other)?;
    let result = run_script("assembly.ts", &payload).map_err(AssemblyError::from);
    let text = result.and_then(|()| Ok(std::fs::read_to_string(&report)?));
    let _ = std::fs::remove_file(&report);
    serde_json::from_str(&text?).map_err(|err| std::io::Error::other(err).into())
}

/// Measures the bodies against the boards' probes and returns the violations of the three rules.
///
/// `parts_dir` is where `check.json` and `preview.glb` are written; without it nothing is
/// written. No bodies means nothing to measure, so bun is not needed.
pub(super) fn clearance(
    ctx: &Context,
    bodies: &[Part],
    parts_dir: Option<&Path>,
) -> Result<Vec<Violation>, AssemblyError> {
    if bodies.is_empty() {
        return Ok(Vec::new());
    }
    let probes = all_probes(ctx)?;
    let preview = parts_dir.map(|dir| dir.join("preview.glb"));
    let response = run_measure(bodies, &probes, preview.as_deref())?;
    if let Some(name) = response.invalid.first() {
        return Err(AssemblyError::BodyNotSolid { name: name.clone() });
    }
    let found = violations(&probes, &response);
    if let Some(dir) = parts_dir {
        write_report(&dir.join("check.json"), &found)?;
    }
    Ok(found)
}
