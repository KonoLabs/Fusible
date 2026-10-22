//! Names for traces and the synthetic nets they create.

use crate::parser::Trace;

pub(super) fn synthetic_id(trace: &Trace, index: usize) -> String {
    trace
        .connectivity_key
        .clone()
        .or_else(|| trace.trace_id.clone())
        .unwrap_or_else(|| format!("source_trace#{index}"))
}

/// `source_trace "<id>"`, or the trace's position when it has no id.
pub(super) fn trace_label(trace: &Trace, index: usize) -> String {
    match trace.trace_id.as_deref() {
        Some(id) => format!("source_trace \"{id}\""),
        None => format!("source_trace #{index}"),
    }
}
