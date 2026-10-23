//! Combining the failures of several boards into one result.

use super::check::CheckError;
use crate::linter::FuseBlown;

/// Prefixes every violation of a blown fuse with the board that raised it, when there is one.
pub(super) fn label(error: CheckError, board: Option<&str>) -> CheckError {
    match (error, board) {
        (CheckError::FuseBlown(mut blown), Some(board)) => {
            for violation in &mut blown.violations {
                violation.message = format!("[{board}] {}", violation.message);
            }
            CheckError::FuseBlown(blown)
        }
        (other, _) => other,
    }
}

/// Folds `next` into `first`: violations merge, and a tool error wins over violations.
///
/// The exit code must say "tool error" when one happened, so a blown-fuse report that loses
/// to a tool error is handed to `report` instead of being dropped.
pub(super) fn merge(
    first: Option<CheckError>,
    next: Option<CheckError>,
    report: &mut impl FnMut(FuseBlown),
) -> Option<CheckError> {
    match (first, next) {
        (Some(CheckError::FuseBlown(mut a)), Some(CheckError::FuseBlown(b))) => {
            a.violations.extend(b.violations);
            Some(CheckError::FuseBlown(a))
        }
        (Some(CheckError::FuseBlown(blown)), Some(tool))
        | (Some(tool), Some(CheckError::FuseBlown(blown))) => {
            report(blown);
            Some(tool)
        }
        (first, next) => first.or(next),
    }
}
