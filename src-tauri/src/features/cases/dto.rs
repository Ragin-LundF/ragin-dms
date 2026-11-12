//! The IPC types of the cases screen; names start with the area.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::document::dto::LinkedDocument;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum CaseStatusDto {
    Open,
    Waiting,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum CaseDirectionDto {
    Received,
    Sent,
}

/// The two lists of the screen's segmented button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum CasesFilterDto {
    Active,
    Done,
}

/// A case's status pill: `label` such as "Action needed" or "Waiting"; `attention` when a follow-up is due.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CaseStatusPillDto {
    pub status: CaseStatusDto,
    pub label: String,
    pub attention: bool,
}

/// One case in the list: `next_step` such as "Send the receipts by 2026-11-30" or "Waiting for an answer for 24
/// days".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CaseRowDto {
    pub id: String,
    pub title: String,
    pub correspondent: Option<String>,
    pub pill: CaseStatusPillDto,
    pub next_step: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum CaseTimelineKindDto {
    Received,
    Sent,
    FollowUp,
    Note,
}

/// One step on the timeline, oldest first: a document in or out (`document_id`), a follow-up (`follow_up_id`,
/// `done`, `due` from its date on) or a note. `marker` is the line above the title, such as "2026-11-04 · received".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CaseTimelineItemDto {
    pub kind: CaseTimelineKindDto,
    pub date: String,
    pub marker: String,
    pub title: String,
    pub detail: Option<String>,
    pub document_id: Option<String>,
    pub follow_up_id: Option<String>,
    pub done: bool,
    pub due: bool,
}

/// What a letter asked the person to do, as the AI read it: the letter and the open follow-ups made from it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CaseRequestDto {
    pub document_id: String,
    pub document_name: String,
    pub actions: Vec<String>,
}

/// The documents and recipient of "Draft reminder".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CaseLetterDto {
    pub case_id: String,
    pub document_ids: Vec<String>,
    pub recipient: Option<String>,
}

/// The case shown on the right: `summary` such as "Tax office · 3 entries".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CaseDetailDto {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub pill: CaseStatusPillDto,
    pub requests: Vec<CaseRequestDto>,
    pub timeline: Vec<CaseTimelineItemDto>,
    pub reminder: CaseLetterDto,
}

/// The screen: the counts of both lists, the chosen list, and the chosen case (or the first of the list).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CasesViewDto {
    pub active_count: u32,
    pub done_count: u32,
    pub cases: Vec<CaseRowDto>,
    pub detail: Option<CaseDetailDto>,
}

/// A case to pick from, such as in "Add to case…".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CaseChoiceDto {
    pub id: String,
    pub title: String,
    pub status: CaseStatusDto,
}

/// The quick look's "Cases" row: the cases the document is in, the active cases it could join (the
/// correspondent's first), and whether it has links to start a case from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CasesOfDocumentDto {
    pub cases: Vec<CaseChoiceDto>,
    pub candidates: Vec<CaseChoiceDto>,
    pub has_links: bool,
}

/// A follow-up to add: `date` as YYYY-MM-DD.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CasesFollowUpDto {
    pub text: String,
    pub date: String,
}

/// The case part of a review: what the letter asks for (each with its proposed follow-up date), documents that
/// match the request, the active case of the same correspondent (`pre_checked` when it waits for an answer) and
/// the title of a new case.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CasesReviewDto {
    pub requested: Vec<CasesFollowUpDto>,
    pub matches: Vec<LinkedDocument>,
    pub open_case: Option<CaseChoiceDto>,
    pub pre_checked: bool,
    pub new_title: String,
}

/// What the approval of a review also does with cases: start a case or add the document to one, with the checked
/// follow-ups.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum CasesReviewChoiceDto {
    #[serde(rename_all = "camelCase")]
    NewCase { title: String, follow_ups: Vec<CasesFollowUpDto> },
    #[serde(rename_all = "camelCase")]
    AddToCase { case_id: String, follow_ups: Vec<CasesFollowUpDto> },
}

/// A new case from the screen or from the quick look's documents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CasesNewDto {
    pub title: String,
    pub correspondent: Option<String>,
    pub document_ids: Vec<String>,
}

/// After "Save as PDF" in the letter dialog: the letter goes into `case_id` as sent, or into a new case with its
/// source documents when `start_case`, and gets a follow-up `follow_up_days` from today.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CasesFileLetterDto {
    pub letter_id: String,
    pub source_ids: Vec<String>,
    pub case_id: Option<String>,
    pub start_case: bool,
    pub follow_up_days: Option<u16>,
}
