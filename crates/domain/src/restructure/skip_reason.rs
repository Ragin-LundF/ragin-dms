/// Why a restructure leaves a document where it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkipReason {
    MissingCategory,
    MissingDate,
    MissingCorrespondent,
    MissingField,
    /// Inbox documents are filed one by one, never restructured.
    InInbox,
}
