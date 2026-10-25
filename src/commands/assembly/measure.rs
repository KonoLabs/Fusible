//! The measure step: probes against bodies in one OpenCascade run, then the rules.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use super::atomic::staging_path;
use super::parts::{Context, Part};
use super::place::IDENTITY;
use super::probe::{probes, Probe};
use super::probe_warnings::warnings;
use super::rules::{violations, write_report, Response, Violation, EPS_VOLUME};
use super::rules_validate::validate;
use super::{blow_if_any, AssemblyError, FORMATS};
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
        for line in warnings(name, &kit) {
            eprintln!("warning: {line}");
        }
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

pub(super) fn remove_stale(parts_dir: &Path, out_dir: &Path) {
    let mut files = vec![parts_dir.join("check.json"), parts_dir.join("preview.glb")];
    files.extend(
        FORMATS
            .iter()
            .map(|f| out_dir.join(format!("assembly.{f}"))),
    );
    for file in files {
        let _ = std::fs::remove_file(file);
    }
}

pub(super) fn report_clearance(
    ctx: &Context,
    bodies: &[Part],
    parts_dir: &Path,
) -> Result<(), AssemblyError> {
    blow_if_any(&clearance(ctx, bodies, Some(parts_dir))?)?;
    if !bodies.is_empty() {
        println!("⚡ Fuse intact — the bodies clear the boards.\n");
    }
    Ok(())
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
        if let Some(dir) = parts_dir {
            std::fs::create_dir_all(dir)?;
            write_report(&dir.join("check.json"), &[])?;
        }
        return Ok(Vec::new());
    }
    let probes = all_probes(ctx)?;
    let preview = parts_dir.map(|dir| dir.join("preview.glb"));
    let staging = preview.as_deref().map(staging_path);
    let measured = run_measure(bodies, &probes, staging.as_deref());
    if let (Some(staging), Some(preview), Ok(_)) = (&staging, &preview, &measured) {
        if staging.exists() {
            std::fs::rename(staging, preview)?;
        }
    }
    if let Some(staging) = &staging {
        let _ = std::fs::remove_file(staging);
    }
    let response = measured?;
    if let Some(name) = response.invalid.first() {
        return Err(AssemblyError::BodyNotSolid { name: name.clone() });
    }
    validate(&probes, &response).map_err(|reason| AssemblyError::BadMeasurement { reason })?;
    let found = violations(&probes, &response);
    if let Some(dir) = parts_dir {
        write_report(&dir.join("check.json"), &found)?;
    }
    Ok(found)
}
