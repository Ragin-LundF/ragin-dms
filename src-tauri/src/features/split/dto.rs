//! What the split dialog shows and sends back. Pages count from 0.

use serde::{Deserialize, Serialize};
use specta::Type;

/// One document of the stack with what the AI read about it. `category_id` names an existing category,
/// `new_category` one the AI would create; `category_label` is only for showing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SplitPartDto {
    pub first_page: u32,
    pub last_page: u32,
    pub title: Option<String>,
    pub category_id: Option<String>,
    pub new_category: Option<String>,
    pub category_label: Option<String>,
    pub correspondent: Option<String>,
    pub document_date: Option<String>,
    /// How sure the AI is, in percent.
    pub confidence: Option<u32>,
}

/// The AI's proposal for a stack; `note` says why the AI's parts were not usable and the stack shows as one part.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SplitProposalDto {
    pub document_id: String,
    pub name: String,
    pub page_count: u32,
    pub model: String,
    pub parts: Vec<SplitPartDto>,
    pub note: Option<String>,
}

/// The cuts the user confirmed with "Split and review".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SplitApplyInputDto {
    pub document_id: String,
    pub parts: Vec<SplitPartDto>,
    pub model: String,
    pub trash_original: bool,
}
