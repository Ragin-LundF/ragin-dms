//! What the library screen shows, derived from the catalog (Kotlin `LibraryUiStateModel` without the UI-only parts).

use serde::{Deserialize, Serialize};
use specta::Type;

use super::params::{LibraryFilterDto, LibraryKindDto};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStateDto {
    pub title: String,
    pub counts: LibraryCountsDto,
    /// The category tree, for the bulk bar's menu and the category actions.
    pub categories: Vec<LibraryCategoryDto>,
    /// The saved view's own conditions, then the active filters.
    pub tokens: Vec<LibraryTokenDto>,
    pub suggestions: Vec<LibraryTokenDto>,
    /// What each facet chip offers, in chip order, counted on the view before filters and search narrow it.
    pub facets: Vec<LibraryFacetOptionsDto>,
    /// Shown only on "All documents" without a filter or search text.
    pub tiles: Vec<CategoryTileDto>,
    /// Every shown document in shown order (sorted, not grouped): quick look walks it, Shift-click ticks ranges in it.
    pub order: Vec<String>,
    pub groups: Vec<LibraryGroupDto>,
    pub smart_folders: Vec<LibrarySavedViewDto>,
    /// Why the view cannot be saved or exported as one query.
    pub query_problem: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryCountsDto {
    pub all: u32,
    pub inbox: u32,
    pub needs_review: u32,
    pub added_this_month: u32,
    pub folders: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryCategoryDto {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub depth: u32,
    /// Documents with exactly this category; a category in use cannot be deleted.
    pub document_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryTokenDto {
    pub key: String,
    pub value: String,
    /// `null` for a saved view's own condition, which cannot be removed.
    pub filter: Option<LibraryFilterDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LibraryFacetDto {
    Category,
    Tag,
    Year,
    Correspondent,
    Type,
    Payment,
    Folder,
    Tax,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFacetOptionsDto {
    pub facet: LibraryFacetDto,
    pub options: Vec<LibraryFacetOptionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFacetOptionDto {
    pub filter: LibraryFilterDto,
    pub label: String,
    pub count: u32,
    pub depth: u32,
    /// "Without a category", drawn italic in the warning tint.
    pub missing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CategoryTileDto {
    pub category_id: String,
    pub name: String,
    pub count: u32,
    pub kinds: Vec<LibraryKindDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryGroupDto {
    pub title: String,
    pub rows: Vec<LibraryRowDto>,
}

/// One document as the grid and the list show it.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryRowDto {
    pub id: String,
    pub path: String,
    pub file_name: String,
    /// The folder it lies in; `null` at the top level.
    pub folder: Option<String>,
    pub title: Option<String>,
    pub kind: LibraryKindDto,
    pub category_name: Option<String>,
    pub correspondent: Option<String>,
    /// `YYYY-MM-DD`.
    pub document_date: Option<String>,
    /// Sorted ignoring case.
    pub tags: Vec<String>,
    /// The amount formatted, such as `12.50 EUR`.
    pub amount: Option<String>,
    /// `YYYY-MM-DD` of the first due-date field.
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySavedViewDto {
    pub id: String,
    pub name: String,
    /// `null` while the view needs the documents' text.
    pub count: Option<u32>,
    /// It names a category, tag or field that no longer exists.
    pub invalid: bool,
}
