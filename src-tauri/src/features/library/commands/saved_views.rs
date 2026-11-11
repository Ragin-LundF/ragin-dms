//! Saved views (smart folders): saved searches shown in the sidebar, kept in the catalog's settings (Kotlin
//! `SmartFolders`). Every change runs as "Saving the saved views" and reports "Saved {name}", "Deleted {name}" or
//! "Added the standard saved views". Each returns whether the change started (`false`: nothing to do).

use dms_application::port::system::IdGenerator;
use dms_application::search::SavedSearchPresets;
use dms_domain::catalog::Catalog;
use dms_domain::query::{SavedSearch, SavedSearchId};
use dms_domain::settings::CatalogSettings;
use dms_domain::taxonomy::TagName;
use dms_domain::text;

use super::moment;
use super::operation::start;
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::features::library::dto::params::{LibraryParamsDto, parse_saved_search_id};
use crate::features::library::view::state::current_query;
use crate::messages;
use crate::session::Lane;

/// Saves the view of `params` (captured when "Save view" was clicked) under `name`; a blank name does nothing.
#[tauri::command]
#[specta::specta]
pub async fn library_save_view(ctx: AppState<'_>, params: LibraryParamsDto, name: String) -> Result<bool, IpcError> {
    let params = params.parse()?;
    let catalog = ctx.require_catalog()?;
    let Some(name) = non_blank(&name) else {
        return Ok(false);
    };
    let query = current_query(&catalog, &params, &moment(&ctx)?)
        .ok_or_else(|| IpcError::invalid("This view cannot be saved"))?;
    let id = SavedSearchId::new(uuid::Uuid::new_v4().to_string()).map_err(IpcError::internal)?;
    let search = SavedSearch { id, name, query };
    change(&ctx, format!("Saved {}", search.name), move |settings| settings.with_search(search.clone()))
}

/// Replaces a saved view's query with what the library shows now, without asking.
#[tauri::command]
#[specta::specta]
pub async fn library_replace_saved_view(
    ctx: AppState<'_>,
    id: String,
    params: LibraryParamsDto,
) -> Result<bool, IpcError> {
    let id = parse_saved_search_id(&id)?;
    let params = params.parse()?;
    let catalog = ctx.require_catalog()?;
    let Some(search) = find(&catalog, &id) else {
        return Ok(false);
    };
    let query = current_query(&catalog, &params, &moment(&ctx)?)
        .ok_or_else(|| IpcError::invalid("This view cannot be saved"))?;
    let replaced = SavedSearch { query, ..search };
    change(&ctx, format!("Saved {}", replaced.name), move |settings| settings.with_search(replaced.clone()))
}

/// Renames a saved view; a blank name does nothing.
#[tauri::command]
#[specta::specta]
pub async fn library_rename_saved_view(ctx: AppState<'_>, id: String, name: String) -> Result<bool, IpcError> {
    let id = parse_saved_search_id(&id)?;
    let catalog = ctx.require_catalog()?;
    let (Some(search), Some(name)) = (find(&catalog, &id), non_blank(&name)) else {
        return Ok(false);
    };
    let renamed = SavedSearch { name, ..search };
    change(&ctx, format!("Saved {}", renamed.name), move |settings| settings.with_search(renamed.clone()))
}

/// Deletes a saved view; the screen asked first. No document changes.
#[tauri::command]
#[specta::specta]
pub async fn library_delete_saved_view(ctx: AppState<'_>, id: String) -> Result<bool, IpcError> {
    let id = parse_saved_search_id(&id)?;
    let catalog = ctx.require_catalog()?;
    let Some(search) = find(&catalog, &id) else {
        return Ok(false);
    };
    change(&ctx, format!("Deleted {}", search.name), move |settings| settings.without_search(&id))
}

/// Adds "Untagged", "Uncategorized", "Tax {year}" and "Due in 30 days" where no saved view has the name yet.
#[tauri::command]
#[specta::specta]
pub async fn library_add_standard_saved_views(ctx: AppState<'_>) -> Result<bool, IpcError> {
    let catalog = ctx.require_catalog()?;
    let wanted = TagName::of("Tax").map_err(IpcError::internal)?;
    let tax = catalog.tags().iter().find(|tag| **tag == wanted).cloned().unwrap_or(wanted);
    let presets = SavedSearchPresets::of(moment(&ctx)?.today(), &Uuids, &tax);
    change(&ctx, "Added the standard saved views".to_owned(), move |settings| {
        let taken: Vec<String> = settings.saved_searches.iter().map(|search| text::lowercase(&search.name)).collect();
        presets
            .iter()
            .filter(|preset| !taken.contains(&text::lowercase(&preset.name)))
            .fold(settings.clone(), |current, preset| current.with_search(preset.clone()))
    })
}

/// Saves the catalog settings `edit` makes as "Saving the saved views"; a busy library does nothing.
fn change(
    ctx: &AppState<'_>,
    done: String,
    edit: impl Fn(&CatalogSettings) -> CatalogSettings + Send + 'static,
) -> Result<bool, IpcError> {
    let started = start(ctx.inner(), "Saving the saved views", Lane::Library, false, move |services, operation| {
        let outcome = services.documents.change_settings.run("Saved views", &edit);
        operation.session().report(messages::of(&outcome, &done));
    });
    match started {
        Ok(()) => Ok(true),
        Err(error) if error.code == "busy" => Ok(false),
        Err(error) => Err(error),
    }
}

fn find(catalog: &Catalog, id: &SavedSearchId) -> Option<SavedSearch> {
    catalog.settings().saved_searches.iter().find(|search| &search.id == id).cloned()
}

fn non_blank(raw: &str) -> Option<String> {
    Some(text::trim(raw)).filter(|name| !name.is_empty()).map(str::to_owned)
}

/// New ids for saved views, as Kotlin's `UUID.randomUUID()`.
struct Uuids;

impl IdGenerator for Uuids {
    fn new_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
