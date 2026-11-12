use dms_domain::document::DocumentId;
use dms_domain::split::SplitProblem;

use crate::common::OperationOutcome;

/// What splitting a stack did.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // returned once per user action
pub enum ApplySplitOutcome {
    /// The parts are catalogued; `outcome` is that of the split, or of trashing the original when that was chosen.
    /// The parts in `without_suggestion` have no review task yet: the AI said nothing about them.
    Split {
        outcome: OperationOutcome,
        parts: Vec<DocumentId>,
        without_suggestion: Vec<DocumentId>,
    },
    /// Nothing was split.
    NotSplit(OperationOutcome),
    InvalidParts(SplitProblem),
    /// One part is the whole stack.
    NothingToSplit,
}
