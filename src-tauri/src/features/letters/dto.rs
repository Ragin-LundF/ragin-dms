//! What the letter dialog exchanges with Rust.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LetterKindDto {
    Cancellation,
    Objection,
    Reminder,
    Request,
    Reply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LetterLanguageDto {
    De,
    En,
}

/// A sender or recipient: name and postal address lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LetterPartyDto {
    pub name: String,
    pub address_lines: Vec<String>,
}

/// What a feature asks a letter for: the documents it refers to, the recipient's name if known (otherwise the
/// correspondent of the newest document) and facts such as a customer number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LetterDraftInputDto {
    pub kind: LetterKindDto,
    pub document_ids: Vec<String>,
    pub recipient: Option<String>,
    pub subject_facts: Vec<String>,
}

/// A draft to edit: `drafted_by_ai` and `origin` (the hint under the letter: who wrote it, and why the template
/// when the AI failed), the sender from "Your details", the folder it is saved to and the folders to choose from
/// ("" is the top level).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LetterDraftDto {
    pub subject: String,
    pub body: String,
    pub sender: Option<LetterPartyDto>,
    pub recipient: LetterPartyDto,
    pub language: LetterLanguageDto,
    pub drafted_by_ai: bool,
    pub origin: String,
    pub folder: String,
    pub folders: Vec<String>,
}

/// The edited letter to save as PDF in `folder`, linked to `document_ids`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LetterSaveInputDto {
    pub document_ids: Vec<String>,
    pub folder: String,
    pub recipient: LetterPartyDto,
    pub subject: String,
    pub body: String,
    pub language: LetterLanguageDto,
}

/// The saved letter's document and its path in the library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LetterSavedDto {
    pub document_id: String,
    pub path: String,
}
