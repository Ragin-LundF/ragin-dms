#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaseStatus {
    Open,
    /// The user waits for an answer since the case's `waiting_since`.
    Waiting,
    Done,
}
