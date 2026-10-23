//! The outcome of a studio run, derived from the exit code contract.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Status {
    Running,
    Ok,
    FuseBlown,
    Error,
}

impl Status {
    pub(super) fn from_code(code: Option<i32>) -> Self {
        match code {
            Some(0) => Self::Ok,
            Some(1) => Self::FuseBlown,
            _ => Self::Error,
        }
    }
}
