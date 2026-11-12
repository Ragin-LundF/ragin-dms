/// Whether a document of a case came in or went out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntryDirection {
    Received,
    Sent,
}
