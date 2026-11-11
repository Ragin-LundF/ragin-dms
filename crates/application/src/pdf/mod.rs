//! PDF page tools, OCR, metadata and rebuilding scans.

mod edit_pages;
mod embed_pdf_metadata;
mod make_all_searchable;
mod make_searchable;
mod merge_documents;
pub mod model;
mod pdf_versions;
pub mod rebuild;
mod scans_without_text;
mod split_document;

pub use edit_pages::EditPages;
pub use embed_pdf_metadata::EmbedPdfMetadata;
pub use make_all_searchable::MakeAllSearchable;
pub use make_searchable::MakeSearchable;
pub use merge_documents::MergeDocuments;
pub use model::{ScansOutcome, SearchableOutcome, SearchableProgress};
pub use pdf_versions::PdfVersions;
pub use scans_without_text::ScansWithoutText;
pub use split_document::SplitDocument;
