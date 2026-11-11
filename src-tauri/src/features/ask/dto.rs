//! What the Ask screen exchanges with Rust.

use serde::{Deserialize, Serialize};
use specta::Type;

/// A source of an answer: `[number]` in the answer text points at this document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CitationDto {
    pub number: u32,
    pub document_id: String,
    pub title: String,
}

/// One question and what came back: the answer with its sources, or a note why there is none. `asking` while
/// the AI still works on it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AskEntryDto {
    pub question: String,
    pub answer: Option<String>,
    pub citations: Vec<CitationDto>,
    pub note: Option<String>,
    pub asking: bool,
}

/// The conversation so far (kept in memory only) and the scopes to ask in: "Whole library", then the smart folders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AskStateDto {
    pub entries: Vec<AskEntryDto>,
    pub scopes: Vec<String>,
}
