//! Validation of a measurement response against the probes that were sent.

use std::collections::HashSet;

use super::probe::Probe;
use super::rules::Response;

/// The only response layout this build reads.
pub const RESPONSE_VERSION: u32 = 1;

/// Checks that `response` answers exactly the probes that were sent.
///
/// A wrong version, a hit on an unknown probe, or a probe without a result means the
/// measurement cannot be trusted, so the caller must fail instead of reporting no violation.
pub fn validate(probes: &[Probe], response: &Response) -> Result<(), String> {
    if response.version != RESPONSE_VERSION {
        return Err(format!(
            "unsupported measurement version {}",
            response.version
        ));
    }
    let known: HashSet<&str> = probes.iter().map(|p| p.id.as_str()).collect();
    if let Some(hit) = response
        .hits
        .iter()
        .find(|h| !known.contains(h.probe.as_str()))
    {
        return Err(format!("hit on unknown probe `{}`", hit.probe));
    }
    let measured: HashSet<&str> = response.measured.iter().map(String::as_str).collect();
    match probes.iter().find(|p| !measured.contains(p.id.as_str())) {
        Some(missing) => Err(format!("no result for probe `{}`", missing.id)),
        None => Ok(()),
    }
}
