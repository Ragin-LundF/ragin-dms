use crate::port::ai::DocumentContent;

/// The document the AI reads: its current file name and its content as text or page images.
#[derive(Debug, Clone)]
pub struct AssistantRequest {
    pub file_name: String,
    pub content: DocumentContent,
}
