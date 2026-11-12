use crate::port::ai::{LineOutcome, PageImage};
use crate::port::assistant::{SplitAnswer, SplitRequest};

/// Lets the AI find where each document of a scanned stack starts. Proposals only; the user decides the cuts.
pub trait SplitProposer: Send + Sync {
    /// The text of one scanned page, read with the vision model, for a stack without a text layer.
    fn read_page(&self, page: &PageImage) -> LineOutcome;

    fn propose(&self, request: &SplitRequest) -> SplitAnswer;
}
