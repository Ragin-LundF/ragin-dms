//! The opt-in inbox: collecting and filing new documents.

mod collect_inbox;
mod configure_inbox;
mod file_inbox_document;
pub mod model;

pub use collect_inbox::CollectInbox;
pub use configure_inbox::ConfigureInbox;
pub use file_inbox_document::FileInboxDocument;
pub use model::CollectInboxOutcome;
