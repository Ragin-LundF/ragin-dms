//! The AI server: classifying, reading documents for it, previews, embeddings and page layouts.

mod ai_server;
mod document_classifier;
mod document_content_reader;
mod document_previewer;
mod embedding_client;
pub mod model;
mod page_layout_reader;

pub use ai_server::AiServer;
pub use document_classifier::DocumentClassifier;
pub use document_content_reader::DocumentContentReader;
pub use document_previewer::DocumentPreviewer;
pub use embedding_client::EmbeddingClient;
pub use model::{
    ClassificationFailure, ClassificationRequest, ClassificationResult, DocumentContent, EmbeddingResult,
    LayoutOutcome, LineOutcome, ModelList, PageImage,
};
pub use page_layout_reader::PageLayoutReader;
