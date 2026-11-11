//! Adding, adopting, moving, renaming, editing, trashing and restoring documents.

mod add_documents;
mod adopt_new_file;
mod edit_document_details;
pub mod model;
mod move_document;
mod restore_document;
mod trash_document;
mod trash_duplicate_file;
mod trash_new_file;

pub use add_documents::AddDocuments;
pub use adopt_new_file::AdoptNewFile;
pub use edit_document_details::EditDocumentDetails;
pub use model::{AddDocumentsOutcome, AddedFile, DetailEdit};
pub use move_document::MoveDocument;
pub use restore_document::RestoreDocument;
pub use trash_document::TrashDocument;
pub use trash_duplicate_file::TrashDuplicateFile;
pub use trash_new_file::TrashNewFile;
