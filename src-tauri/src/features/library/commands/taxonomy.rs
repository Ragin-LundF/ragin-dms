//! Creating, renaming and deleting categories and tags from the facet popovers (Kotlin `TaxonomyController`).
//! Each waits for the catalog edit and returns whether the catalog took it, so the screen changes its view only
//! then; refusals are reported as messages.

use dms_application::catalog::CatalogEdit;
use dms_application::common::OperationOutcome;
use dms_domain::catalog::Catalog;
use dms_domain::taxonomy::TagName;
use dms_domain::text;

use super::operation::wait;
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::features::library::dto::folders::LibraryConfirmationDto;
use crate::features::library::dto::params::{parse_category_id, parse_tag};
use crate::messages;
use crate::session::UiMessage;

/// A new category below `parentId`, or at the top level.
#[tauri::command]
#[specta::specta]
pub async fn library_add_category(
    ctx: AppState<'_>,
    name: String,
    parent_id: Option<String>,
) -> Result<bool, IpcError> {
    let parent_id = parent_id.as_deref().map(parse_category_id).transpose()?;
    let Some(name) = category_name(&ctx, &name) else {
        return Ok(false);
    };
    save(&ctx, CatalogEdit::AddCategory { name, parent_id }).await
}

#[tauri::command]
#[specta::specta]
pub async fn library_rename_category(ctx: AppState<'_>, category_id: String, name: String) -> Result<bool, IpcError> {
    let category_id = parse_category_id(&category_id)?;
    let Some(name) = category_name(&ctx, &name) else {
        return Ok(false);
    };
    save(&ctx, CatalogEdit::RenameCategory { category_id, name }).await
}

/// Deletes an unused category; the screen asked first.
#[tauri::command]
#[specta::specta]
pub async fn library_remove_category(ctx: AppState<'_>, category_id: String) -> Result<bool, IpcError> {
    let category_id = parse_category_id(&category_id)?;
    save(&ctx, CatalogEdit::RemoveCategory { category_id }).await
}

/// Renames a tag in the catalog and every document; a name that cannot be a tag is reported.
#[tauri::command]
#[specta::specta]
pub async fn library_rename_tag(ctx: AppState<'_>, tag: String, name: String) -> Result<bool, IpcError> {
    let from = parse_tag(&tag)?;
    let Some(to) = TagName::parse(&name) else {
        ctx.session.report(UiMessage::error("Not a valid tag"));
        return Ok(false);
    };
    save(&ctx, CatalogEdit::RenameTag { from, to }).await
}

/// The question before removing a tag, naming every document that has it.
#[tauri::command]
#[specta::specta]
pub async fn library_remove_tag_request(ctx: AppState<'_>, tag: String) -> Result<LibraryConfirmationDto, IpcError> {
    let tag = parse_tag(&tag)?;
    let catalog = ctx.require_catalog()?;
    Ok(remove_tag_confirmation(&catalog, &tag))
}

#[tauri::command]
#[specta::specta]
pub async fn library_remove_tag(ctx: AppState<'_>, tag: String) -> Result<bool, IpcError> {
    let tag = parse_tag(&tag)?;
    save(&ctx, CatalogEdit::RemoveTag { tag }).await
}

/// Saves `edit` as "Saving the catalog" and reports a refusal; success says nothing, as in Kotlin. A busy library
/// does nothing.
async fn save(ctx: &AppState<'_>, edit: CatalogEdit) -> Result<bool, IpcError> {
    let done = wait(ctx.inner(), "Saving the catalog", move |services, operation| {
        let outcome = services.documents.edit_catalog.run(&edit);
        let done = matches!(outcome, OperationOutcome::Done { .. });
        if !done {
            operation.session().report(messages::of(&outcome, ""));
        }
        done
    });
    match done.await {
        Err(error) if error.code == "busy" => Ok(false),
        other => other,
    }
}

/// The trimmed name, or `None` after reporting "The name is empty".
fn category_name(ctx: &AppState<'_>, raw: &str) -> Option<String> {
    let name = text::trim(raw);
    if name.is_empty() {
        ctx.session.report(UiMessage::error("The name is empty"));
        return None;
    }
    Some(name.to_owned())
}

fn remove_tag_confirmation(catalog: &Catalog, tag: &TagName) -> LibraryConfirmationDto {
    let tagged: Vec<String> = catalog
        .documents()
        .values()
        .filter(|document| document.tags.contains(tag))
        .map(|document| document.path.value().to_owned())
        .collect();
    LibraryConfirmationDto {
        title: format!("Remove tag \"{}\"", tag.value()),
        message: format!(
            "The tag is removed from the catalog and from {} document(s). The files stay as they are.",
            tagged.len()
        ),
        affected: tagged,
        confirm_label: "Remove tag".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::library::test_catalog::{TestCatalog, tag};

    #[test]
    fn removing_a_tag_names_every_tagged_document() {
        let catalog = TestCatalog::new()
            .document("a.pdf", |document| {
                document.tags(&["Tax"]);
            })
            .document("b.pdf", |_| {})
            .build();
        let confirmation = remove_tag_confirmation(&catalog, &tag("tax"));
        assert_eq!(confirmation.affected, ["a.pdf"]);
        assert_eq!(
            confirmation.message,
            "The tag is removed from the catalog and from 1 document(s). The files stay as they are."
        );
    }
}
