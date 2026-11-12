//! Splitting a scanned stack: the AI proposes where each document starts, the user sets the cuts, and every part
//! waits for review with the AI's values as a suggestion.

mod apply_split;
pub mod model;
mod propose_split;
mod stack_pages;

pub use apply_split::ApplySplit;
pub use model::{ApplySplitInput, ApplySplitOutcome, PageSource, ProposeSplitOutcome, SplitProgress};
pub use propose_split::ProposeSplit;
