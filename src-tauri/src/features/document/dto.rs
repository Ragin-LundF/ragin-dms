//! What the document area exchanges with the UI: the quick look of one document with its details, the review of
//! an AI suggestion, and the assistant's answers.

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

/// Everything the quick look shows of one document (Kotlin `QuickLookSheet` and what it derived).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct QuickLook {
    pub document_id: String,
    pub path: String,
    pub file_name: String,
    pub title: Option<String>,
    /// `pdf`, `image`, `office` or `other`.
    pub kind: String,
    /// `false` for kinds the AI never reads.
    pub readable_by_ai: bool,
    /// The folder the preview comes from; `None` without a primary folder.
    pub primary_folder_id: Option<String>,
    pub facts: QuickFacts,
    /// The chips of the suggestion that waits for review, in the order "Accept all" applies them.
    pub suggestion: Option<Vec<String>>,
    /// Every category for the category menu, subcategories below their parent.
    pub categories: Vec<CategoryChoice>,
    /// The library's tags, sorted, for the tag suggestions.
    pub library_tags: Vec<String>,
    /// What "Move…" starts with: the current folder, or `/` at the top level.
    pub folder: String,
    pub details: DocumentDetails,
}

/// The key facts; `None` where there is no value. `amount` and `due` are `None` while the library defines no such
/// field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct QuickFacts {
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub correspondent: Option<String>,
    pub document_date: Option<String>,
    pub amount: Option<FieldFact>,
    pub due: Option<FieldFact>,
}

/// A custom field among the key facts; a deadline has a `chip` such as "in 11 days", `urgent` once it is within its
/// lead time or past.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FieldFact {
    pub label: String,
    pub value: Option<String>,
    pub chip: Option<String>,
    pub urgent: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CategoryChoice {
    pub id: String,
    pub name: String,
    pub depth: u32,
}

/// The values of the quick look a user types in place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum QuickFact {
    Title,
    Correspondent,
    Date,
    Amount,
    Due,
    Tag,
}

/// The folded part of the quick look: note, links, fields, rule and retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDetails {
    pub note: Option<String>,
    pub links: Vec<LinkedDocument>,
    /// One editor per field definition; empty while the library has none.
    pub fields: Vec<FieldEditor>,
    /// Why "Create rule from this document" cannot be used.
    pub rule_disabled_reason: Option<String>,
    pub retention: RetentionEditor,
}

/// A document to link to, or a linked one with the link's label.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LinkedDocument {
    pub document_id: String,
    pub name: String,
    pub path: String,
    pub label: Option<String>,
}

/// The editor of one custom field: its label with the hint, such as "Amount (amount and currency)", and the value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FieldEditor {
    pub key: String,
    pub label: String,
    pub value: String,
}

/// One typed value of a field editor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TypedField {
    pub key: String,
    pub value: String,
}

/// The retention editor: the status line, the saved keep-until date and the period of the document's category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RetentionEditor {
    pub status: String,
    pub keep_until: String,
    pub category: Option<CategoryRetention>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRetention {
    pub id: String,
    /// `Keep documents of "{name}" for … years`.
    pub label: String,
    pub years: String,
}

/// The question before trashing a document: one entry per library folder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TrashQuestion {
    pub title: String,
    pub message: String,
    pub affected: Vec<String>,
    pub confirm_label: String,
}

/// The review of the suggestion for one document (Kotlin `ReviewStateModel`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub document_id: String,
    /// The document's path, as the title and the messages name it.
    pub document_name: String,
    /// "From {model} · NN % sure. Only the checked values are applied."
    pub source: String,
    pub rows: Vec<ReviewRow>,
}

/// One suggested value. `key` is `title`, `documentDate`, `correspondent`, `category`, `tags`, `fileName`,
/// `targetFolder` or `field:{fieldKey}`. `hint` is the fixed part of the line under the field; the category row adds
/// "will be created" while typing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ReviewRow {
    pub key: String,
    pub label: String,
    pub value: String,
    pub accepted: bool,
    pub current: String,
    pub options: Vec<String>,
    pub hint: Option<String>,
}

/// One row as the user left it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ReviewRowDraft {
    pub key: String,
    pub accepted: bool,
    pub value: String,
}

/// A summary by the AI, or why there is none.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SummaryAnswer {
    pub text: Option<String>,
    pub model: Option<String>,
    pub problem: Option<String>,
}

/// One message of a conversation about a document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    /// `user` or `assistant`.
    pub role: ChatSpeaker,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ChatSpeaker {
    User,
    Assistant,
}

/// The AI's answer to the last question, or why there is none.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChatAnswer {
    pub text: Option<String>,
    pub problem: Option<String>,
}

/// An AI run made a suggestion for `document_id`: the window opens its review (Kotlin `review.open` from
/// `classifyWithAi`/`tagWithAi`).
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct OpenReviewEvent {
    pub document_id: String,
}
