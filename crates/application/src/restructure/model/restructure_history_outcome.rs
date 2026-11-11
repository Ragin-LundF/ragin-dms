#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestructureHistoryOutcome {
    /// The names of the restructure records, newest first; each can be reversed.
    Listed(Vec<String>),
    LibraryNotOpen,
}
