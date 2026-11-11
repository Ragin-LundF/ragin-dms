//! Reads of the library screen and the folder view, and the panel layout.

use std::collections::HashSet;

use dms_application::folder::FoldersOutcome;
use dms_application::port::state::{LayoutPreferences, Preferences};
use dms_application::search::{FindOutcome, SearchOutcome};
use dms_domain::document::DocumentId;
use dms_domain::library::RelativePath;
use dms_domain::task::OpenTask;
use dms_domain::text;

use super::moment;
use crate::commands::AppState;
use crate::context::AppContext;
use crate::dto::IpcError;
use crate::features::library::dto::folders::{
    LibraryFolderEntryDto, LibraryFolderViewDto, LibraryFolderViewParamsDto, LibraryLayoutDto,
};
use crate::features::library::dto::params::{LibraryParamsDto, parse_path};
use crate::features::library::dto::state::LibraryStateDto;
use crate::features::library::model::View;
use crate::features::library::view::filter::Matches;
use crate::features::library::view::folders::{FolderViewParams, picker_rows, present_folder_view, proposed_name};
use crate::features::library::view::state::present_library;
use crate::messages;
use crate::session::reporting::blocking;

/// The library screen for `params`. `contentQuery` is the search text, debounced, while the content search is on.
#[tauri::command]
#[specta::specta]
pub async fn library_state(
    ctx: AppState<'_>,
    params: LibraryParamsDto,
    content_query: Option<String>,
) -> Result<LibraryStateDto, IpcError> {
    let params = params.parse()?;
    let ctx = ctx.inner().clone();
    blocking(move || {
        let catalog = ctx.require_catalog()?;
        let services = ctx.require_services()?;
        let moment = moment(&ctx)?;
        // The content search only reads, so it never waits for a busy library (Kotlin `ContentSearch`).
        let content: Option<HashSet<DocumentId>> = content_query.filter(|query| !text::is_blank(query)).map(|query| {
            match services.library.search_contents.run(text::trim(&query)) {
                SearchOutcome::Found(ids) => ids.into_iter().collect(),
                SearchOutcome::LibraryNotOpen => HashSet::new(),
            }
        });
        // A saved view whose query needs the documents' text finds its matches by content (Kotlin `SmartFolders`).
        let smart: Option<HashSet<DocumentId>> = match &params.view {
            View::SmartFolder(id) => catalog
                .settings()
                .saved_searches
                .iter()
                .find(|search| &search.id == id)
                .filter(|search| search.query.needs_content())
                .map(|search| match services.library.find_documents.run(&search.query) {
                    FindOutcome::Found(documents) => documents.into_iter().map(|document| document.id).collect(),
                    FindOutcome::LibraryNotOpen => HashSet::new(),
                }),
            _ => None,
        };
        let review = review_ids(&ctx);
        let matches = Matches { text: content.as_ref(), query: smart.as_ref(), review: Some(&review) };
        Ok(present_library(&catalog, &params, &matches, &moment))
    })
    .await
}

/// The documents whose AI suggestion waits for review, from the session's dashboard (Kotlin `PendingReviews`).
fn review_ids(ctx: &AppContext) -> HashSet<DocumentId> {
    ctx.dashboard()
        .map(|dashboard| {
            dashboard
                .tasks
                .iter()
                .filter_map(|task| match task {
                    OpenTask::ReviewSuggestion(suggestion) => Some(suggestion.document_id.clone()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The folders on disk in the primary folder, also empty ones the catalog cannot know (Kotlin `DiskFolders`). A
/// failed read is an error, so the screen keeps the folders it knew.
#[tauri::command]
#[specta::specta]
pub async fn library_disk_folders(ctx: AppState<'_>) -> Result<Vec<String>, IpcError> {
    let ctx = ctx.inner().clone();
    blocking(move || match ctx.require_services()?.documents.read_folders.run() {
        FoldersOutcome::Read(folders) => Ok(folders.iter().map(|folder| folder.value().to_owned()).collect()),
        FoldersOutcome::LibraryNotOpen => Ok(Vec::new()),
        FoldersOutcome::StorageFailed(error) => Err(IpcError::new("storage", messages::failure(error.reason))),
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn library_folder_view(
    ctx: AppState<'_>,
    params: LibraryFolderViewParamsDto,
) -> Result<LibraryFolderViewDto, IpcError> {
    let params = FolderViewParams {
        open: parse_path(&params.open)?,
        expanded: parse_paths(&params.expanded)?.into_iter().collect(),
        query: params.query,
        folders_on_disk: parse_paths(&params.folders_on_disk)?,
    };
    let catalog = ctx.require_catalog()?;
    Ok(present_folder_view(&catalog, &params))
}

/// The rows of a "Move to…" picker below its "Library" row, subfolders while `expanded`.
#[tauri::command]
#[specta::specta]
pub async fn library_folder_tree(
    ctx: AppState<'_>,
    folders_on_disk: Vec<String>,
    expanded: Vec<String>,
) -> Result<Vec<LibraryFolderEntryDto>, IpcError> {
    let on_disk = parse_paths(&folders_on_disk)?;
    let expanded = parse_paths(&expanded)?.into_iter().collect();
    let catalog = ctx.require_catalog()?;
    Ok(picker_rows(&catalog, &on_disk, &expanded))
}

/// "New folder", or the next free "New folder N" in `parent`.
#[tauri::command]
#[specta::specta]
pub async fn library_proposed_folder_name(
    ctx: AppState<'_>,
    parent: String,
    folders_on_disk: Vec<String>,
) -> Result<String, IpcError> {
    let parent = parse_path(&parent)?;
    let on_disk = parse_paths(&folders_on_disk)?;
    let catalog = ctx.require_catalog()?;
    Ok(proposed_name(&catalog, &on_disk, &parent))
}

/// The stored sidebar and folder tree widths and folds (the settings file clamps a stored width when it loads).
#[tauri::command]
#[specta::specta]
pub async fn library_layout(ctx: AppState<'_>) -> Result<LibraryLayoutDto, IpcError> {
    let ctx = ctx.inner().clone();
    blocking(move || {
        let layout = load(&ctx)?.layout;
        Ok(LibraryLayoutDto {
            sidebar_width: layout.sidebar_width,
            sidebar_folded: layout.sidebar_folded,
            folder_tree_width: layout.folder_tree_width,
            folder_tree_hidden: layout.folder_tree_hidden,
        })
    })
    .await
}

/// Stores the panel layout when a drag ends or a panel folds (Kotlin `LayoutPreferences.update`). It is written
/// directly, not as an operation, so resizing is kept while the library is busy.
#[tauri::command]
#[specta::specta]
pub async fn library_set_layout(ctx: AppState<'_>, layout: LibraryLayoutDto) -> Result<(), IpcError> {
    let ctx = ctx.inner().clone();
    blocking(move || {
        let sidebar = LayoutPreferences::SIDEBAR_WIDTHS;
        let tree = LayoutPreferences::FOLDER_TREE_WIDTHS;
        let layout = LayoutPreferences {
            sidebar_width: layout.sidebar_width.clamp(*sidebar.start(), *sidebar.end()),
            sidebar_folded: layout.sidebar_folded,
            folder_tree_width: layout.folder_tree_width.clamp(*tree.start(), *tree.end()),
            folder_tree_hidden: layout.folder_tree_hidden,
        };
        ctx.write_preferences(|preferences| Preferences { layout, ..preferences })
            .map(|_| ())
            .map_err(|failure| IpcError::new("storage", failure))
    })
    .await
}

fn load(ctx: &AppContext) -> Result<Preferences, IpcError> {
    ctx.require_services()?
        .library
        .settings
        .load_preferences()
        .map_err(|error| IpcError::new("storage", messages::failure(error.reason)))
}

pub(super) fn parse_paths(raw: &[String]) -> Result<Vec<RelativePath>, IpcError> {
    raw.iter().map(|path| parse_path(path)).collect()
}
