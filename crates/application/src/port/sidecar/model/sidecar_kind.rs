/// Which hidden folder a per-document file belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SidecarKind {
    /// `.dms-text/<id>.txt`: the text search reads.
    Text,
    /// `.dms-index/<id>.vec`: the embeddings Ask reads.
    Index,
}
