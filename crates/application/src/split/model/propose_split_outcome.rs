use dms_domain::split::SplitProposal;

use crate::port::ai::ClassificationFailure;
use crate::port::storage::StorageError;
use crate::split::PageSource;

/// The AI's proposal for a stack, already checked against its pages, or why there is none.
#[derive(Debug, Clone, PartialEq)]
pub enum ProposeSplitOutcome {
    Proposed {
        proposal: SplitProposal,
        model: String,
        page_count: usize,
        source: PageSource,
    },
    LibraryNotOpen,
    NotFound,
    NotPdf,
    /// A stack needs at least two pages.
    TooFewPages,
    /// The user stopped the reading of the pages.
    Cancelled,
    Unreadable(StorageError),
    Failed {
        reason: ClassificationFailure,
        detail: Option<String>,
    },
}
