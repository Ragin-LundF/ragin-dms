/// What to do about a file that was moved outside the tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveDecision {
    /// Keep the new path: move the file in the other folders too and update the catalog.
    Apply,
    /// Move the file back to the path the catalog knows.
    Undo,
}
