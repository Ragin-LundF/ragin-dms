#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyncActionKind {
    Copied,
    Moved,
}
