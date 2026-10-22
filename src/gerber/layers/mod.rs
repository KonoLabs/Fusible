//! Layer builders mapping Circuit JSON elements to RS-274X and Excellon files.

mod board;
mod build;
mod copper;
mod drill;
mod elements;
mod features;
mod font;
mod geometry;
mod holes;
mod math;
mod parse;
mod rounded;
mod shapes;
mod sites;
mod text;

use thiserror::Error;

pub use copper::CopperLayer;
pub use elements::{
    PcbBoardElm, PcbComponentElm, PcbHoleElm, PcbModel, PcbPlatedHoleElm, PcbPoint,
    PcbSilkscreenPathElm, PcbSilkscreenTextElm, PcbSmtPadElm, PcbTraceElm, PcbTracePoint,
    PcbViaElm,
};

/// Reasons a Circuit JSON document cannot be turned into fabrication files.
#[derive(Debug, Error)]
pub enum GerberError {
    #[error("invalid Circuit JSON: {0}")]
    InvalidJson(String),
    #[error("Circuit JSON contains no pcb_board; refusing to emit fabrication files")]
    NoBoard,
    #[error("malformed {kind} at JSON index {index}: {reason}")]
    Malformed {
        kind: String,
        index: usize,
        reason: String,
    },
    #[error("invalid {kind} #{index}: {reason}")]
    Invalid {
        kind: &'static str,
        index: usize,
        reason: String,
    },
}

impl From<GerberError> for std::io::Error {
    fn from(err: GerberError) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
    }
}

/// Wraps a per-element validation message into a [`GerberError::Invalid`] for `kind` #`index`.
fn invalid(kind: &'static str, index: usize) -> impl Fn(String) -> GerberError {
    move |reason| GerberError::Invalid {
        kind,
        index,
        reason,
    }
}
