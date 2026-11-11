//! The folder view, the folder trees of the pickers, the panel layout, and confirmations computed in Rust.

use serde::{Deserialize, Serialize};
use specta::Type;

use super::state::LibraryRowDto;

/// One row of a folder tree, flattened; `documentCount` counts everything below it.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFolderEntryDto {
    pub path: String,
    pub name: String,
    /// 1 for a top-level folder.
    pub depth: u32,
    pub document_count: u32,
    pub has_children: bool,
    pub expanded: bool,
}

/// What the folder view asks for: the open folder, the unfolded tree rows, the search text and the folders on disk.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFolderViewParamsDto {
    pub open: String,
    pub expanded: Vec<String>,
    pub query: String,
    pub folders_on_disk: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFolderViewDto {
    /// "Library" first, then every folder down to the open one.
    pub crumbs: Vec<LibraryCrumbDto>,
    /// The visible tree rows below the "Library" row.
    pub tree: Vec<LibraryFolderEntryDto>,
    pub library_count: u32,
    pub tiles: Vec<LibraryFolderTileDto>,
    /// The documents directly in the open folder matching the search, newest first.
    pub documents: Vec<LibraryRowDto>,
    pub search_placeholder: String,
    pub empty_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryCrumbDto {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFolderTileDto {
    pub path: String,
    pub name: String,
    /// "2 documents · 1 folder", or "Empty".
    pub caption: String,
}

/// The library's panel widths and folds, stored per device (Kotlin `LayoutPreferencesModel`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryLayoutDto {
    pub sidebar_width: i32,
    pub sidebar_folded: bool,
    pub folder_tree_width: i32,
    pub folder_tree_hidden: bool,
}

/// A question before a destructive action, phrased in Rust because it needs the catalog or the disk.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryConfirmationDto {
    pub title: String,
    pub message: String,
    pub affected: Vec<String>,
    pub confirm_label: String,
}
