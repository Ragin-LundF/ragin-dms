//! What the restructure screen exchanges with Rust.

use serde::{Deserialize, Serialize};
use specta::Type;

/// What to restructure: a folder and everything below it, or the documents a smart folder finds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RestructureScopeDto {
    /// `path` is relative; "" is the whole library.
    Folder { path: String },
    /// `id` of the saved search.
    SmartFolder { id: String },
}

/// One entry of the scope picker, named as the picker shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ScopeOptionDto {
    pub label: String,
    pub scope: RestructureScopeDto,
}

/// Whether moved files keep their names or are renamed to "date title" where a document has both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FileNamingDto {
    Keep,
    DateAndTitle,
}

/// The planned moves into one target `folder` ("" is the top level), with up to three `old → new` samples.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MoveGroupDto {
    pub folder: String,
    pub count: u32,
    pub samples: Vec<String>,
}

/// The planned restructure: groups sorted by folder, what stays in place, what is left out by reason
/// ("3 without date") and the moves that got a number for a name clash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlanDto {
    pub groups: Vec<MoveGroupDto>,
    pub unchanged: u32,
    pub skipped: Vec<String>,
    pub renamed: Vec<String>,
}

/// The screen's data: the scopes, the template to start with, the chips, the current plan (until it is applied)
/// and the folders an applied restructure left empty, which the screen offers to remove.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RestructureStateDto {
    pub scopes: Vec<ScopeOptionDto>,
    pub saved_template: Option<String>,
    /// The saved template, or `{category}/{year|Undated}`.
    pub initial_template: String,
    /// The placeholder chips, without braces.
    pub placeholders: Vec<String>,
    pub plan: Option<PlanDto>,
    pub empty_folders: Vec<String>,
}

/// The result of Preview: the plan, or why the template cannot be used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PreviewDto {
    Planned { plan: PlanDto },
    Problem { text: String },
}

/// What Apply would do with the ticked groups: the number of moves and the confirmation's affected lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ApplyPreviewDto {
    pub moves: u32,
    pub affected: Vec<String>,
}
