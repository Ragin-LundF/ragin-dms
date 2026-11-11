//! Documents: ids, content hashes, metadata and the kind of file.

mod classification_source;
mod document_kind;
mod file_name_dates;
pub mod model;

pub use classification_source::ClassificationSource;
pub use document_kind::DocumentKind;
pub use file_name_dates::FileNameDates;
pub use model::{ClassificationRecord, ContentHash, Document, DocumentId, DocumentMetadata};
