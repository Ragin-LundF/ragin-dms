//! Exporting documents into a folder or a ZIP file outside the library (Kotlin `ExportController.export`).

use dms_application::export::{ExportOptions, ExportOutcome, ExportScope};
use dms_domain::catalog::Catalog;
use dms_domain::query::{DateRange, DocumentQuery};
use dms_domain::tax::TaxFilter;
use dms_domain::taxonomy::TagName;
use dms_domain::text;

use super::moment;
use super::operation::wait;
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::features::library::dto::export::{LibraryExportOptionsDto, LibraryExportScopeDto};
use crate::features::library::dto::params::parse_saved_search_id;
use crate::features::library::model::Moment;
use crate::features::library::view::state::current_query;
use crate::messages;
use crate::session::UiMessage;

/// Exports the documents of `scope`, labelled `scopeLabel`, into `directory` as `name`. Returns whether it was
/// exported: a refused or failed export keeps the dialog, so the user can correct it.
#[tauri::command]
#[specta::specta]
pub async fn library_export(
    ctx: AppState<'_>,
    scope: LibraryExportScopeDto,
    scope_label: String,
    directory: String,
    name: String,
    options: LibraryExportOptionsDto,
) -> Result<bool, IpcError> {
    let name = text::trim(&name).to_owned();
    if name.is_empty() {
        return Ok(false);
    }
    let catalog = ctx.require_catalog()?;
    let query = query_of(&catalog, &scope, &moment(&ctx)?)?;
    let tax_year = match scope {
        LibraryExportScopeDto::TaxYear { year } if options.tax_grouping => Some(year),
        LibraryExportScopeDto::TaxYear { .. }
        | LibraryExportScopeDto::View { .. }
        | LibraryExportScopeDto::SmartFolder { .. }
        | LibraryExportScopeDto::Correspondent { .. } => None,
    };
    let options = ExportOptions {
        flatten: options.flatten,
        rename_to_title: options.rename_to_title,
        index: options.index,
        zip: options.zip,
        tax_year,
    };
    let label = format!("Exporting {scope_label}");
    let exported = wait(ctx.inner(), &label, move |services, operation| {
        let outcome = services.organize.export_documents.run(&ExportScope::Query(query), &directory, &name, options);
        operation.session().report(message_of(&outcome));
        matches!(outcome, ExportOutcome::Exported { .. })
    });
    match exported.await {
        Err(error) if error.code == "busy" => Ok(false),
        other => other,
    }
}

fn message_of(outcome: &ExportOutcome) -> UiMessage {
    match outcome {
        ExportOutcome::Exported { count, location } => {
            UiMessage::info(format!("Exported {count} documents to {location}"))
        }
        ExportOutcome::Empty => UiMessage::error("No document matches; nothing was exported"),
        ExportOutcome::InsideLibrary => UiMessage::error("Choose a folder outside the library"),
        ExportOutcome::LibraryNotOpen => UiMessage::error("No library is open"),
        ExportOutcome::Failed(error) => {
            UiMessage::error(format!("The export failed: {}", messages::failure(error.reason)))
        }
        ExportOutcome::InvalidInput(error) => UiMessage::error(error.to_string()),
    }
}

fn query_of(catalog: &Catalog, scope: &LibraryExportScopeDto, moment: &Moment) -> Result<DocumentQuery, IpcError> {
    match scope {
        LibraryExportScopeDto::View { params } => current_query(catalog, &params.parse()?, moment)
            .ok_or_else(|| IpcError::invalid("This view cannot be exported")),
        LibraryExportScopeDto::SmartFolder { id } => {
            let id = parse_saved_search_id(id)?;
            catalog
                .settings()
                .saved_searches
                .iter()
                .find(|search| search.id == id)
                .map(|search| search.query.clone())
                .ok_or_else(|| IpcError::invalid("The saved view no longer exists"))
        }
        LibraryExportScopeDto::TaxYear { year } => {
            DateRange::year(*year).map_err(|error| IpcError::invalid(error.to_string()))?;
            let wanted = TagName::of("Tax").map_err(IpcError::internal)?;
            let tax = catalog.tags().iter().find(|tag| **tag == wanted).cloned().unwrap_or(wanted);
            let filter = TaxFilter { year: Some(*year), area: None, tag_fallback: Some(tax) };
            Ok(DocumentQuery { tax: Some(filter), ..DocumentQuery::default() })
        }
        LibraryExportScopeDto::Correspondent { name } => {
            let name = text::trim(name);
            if name.is_empty() {
                return Err(IpcError::invalid("A correspondent needs a name"));
            }
            Ok(DocumentQuery { correspondent: Some(name.to_owned()), ..DocumentQuery::default() })
        }
    }
}
