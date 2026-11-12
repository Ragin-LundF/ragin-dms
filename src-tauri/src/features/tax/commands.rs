//! The commands of the tax screen, the quick look's tax row, the bulk bar and the dashboard card.
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use dms_application::common::OperationOutcome;
use dms_application::tax::TaxItemChange;
use dms_domain::document::DocumentId;
use dms_domain::field::{FieldType, FieldValues};
use dms_domain::money::Money;
use dms_domain::tax::{TaxAssignment, TaxItemId};
use dms_domain::text;

use super::dto::{
    TaxAssignInputDto, TaxDocumentChoiceDto, TaxDocumentDto, TaxItemChangeDto, TaxProgressDto, TaxYearViewDto,
};
use super::view::amounts::area_of;
use super::view::{document, progress, screen};
use crate::commands::AppState;
use crate::context::AppContext;
use crate::dto::IpcError;
use crate::features::library::commands::operation::{documents_text, wait};
use crate::messages;

/// The checklist of `year` (the default year when null), its progress and amounts per area.
#[tauri::command]
#[specta::specta]
pub async fn tax_year_view(ctx: AppState<'_>, year: Option<i32>) -> Result<TaxYearViewDto, IpcError> {
    let catalog = ctx.require_catalog()?;
    Ok(screen::present_tax_year(&catalog, year, today(&ctx)?))
}

/// Creates or refreshes the stored checklist of `year`; writes only when it changed. `false` while the library is
/// busy; the screen then shows the preview and tries again later.
#[tauri::command]
#[specta::specta]
pub async fn tax_open_year(ctx: AppState<'_>, year: i32) -> Result<bool, IpcError> {
    let label = format!("Preparing the tax year {year}");
    run(&ctx, &label, None, move |services| services.organize.open_tax_year.run(year)).await
}

/// Adds, renames, marks "not needed" or links a document to an item of `year`'s checklist.
#[tauri::command]
#[specta::specta]
pub async fn tax_update_item(ctx: AppState<'_>, year: i32, change: TaxItemChangeDto) -> Result<bool, IpcError> {
    let change = change_of(change)?;
    let done = format!("Updated the tax year {year}");
    run(&ctx, "Updating the tax year", Some(done), move |services| services.organize.update_tax_item.run(year, &change))
        .await
}

/// Assigns documents to a tax year and area, or removes their assignment.
#[tauri::command]
#[specta::specta]
pub async fn tax_assign(ctx: AppState<'_>, input: TaxAssignInputDto) -> Result<bool, IpcError> {
    let ids = input
        .document_ids
        .iter()
        .map(|id| DocumentId::new(id.as_str()).map_err(invalid))
        .collect::<Result<Vec<_>, _>>()?;
    let tax = match (input.year, input.area) {
        (Some(year), Some(area)) => {
            let deductible =
                if input.keep_deductible { None } else { deductible_of(&ctx, input.deductible.as_deref())? };
            Some(TaxAssignment { year, area: area_of(area), deductible })
        }
        (None, None) => None,
        _ => return Err(IpcError::invalid("A tax assignment needs both a year and an area")),
    };
    let done = match &tax {
        Some(tax) => format!("Assigned {} to the tax year {}", documents_text(ids.len()), tax.year),
        None => format!("Removed the tax assignment of {}", documents_text(ids.len())),
    };
    let keep = input.keep_deductible;
    run(&ctx, "Assigning to a tax year", Some(done), move |services| {
        services.organize.assign_tax.run(&ids, tax.as_ref(), keep)
    })
    .await
}

/// The tax row of the quick look for one document.
#[tauri::command]
#[specta::specta]
pub async fn tax_document(ctx: AppState<'_>, document_id: String) -> Result<TaxDocumentDto, IpcError> {
    let catalog = ctx.require_catalog()?;
    let id = DocumentId::new(document_id).map_err(invalid)?;
    let document = catalog.documents().get(&id).ok_or_else(|| IpcError::invalid("The document no longer exists"))?;
    Ok(document::present_document(document, today(&ctx)?))
}

/// The documents "I have it" offers for an item, best candidates first.
#[tauri::command]
#[specta::specta]
pub async fn tax_document_choices(
    ctx: AppState<'_>,
    year: i32,
    item_id: String,
) -> Result<Vec<TaxDocumentChoiceDto>, IpcError> {
    let catalog = ctx.require_catalog()?;
    let item_id = TaxItemId::new(item_id).map_err(invalid)?;
    Ok(document::present_choices(&catalog, year, &item_id))
}

/// The dashboard card "Tax year progress"; null while the library uses no tax assignment.
#[tauri::command]
#[specta::specta]
pub async fn tax_progress(ctx: AppState<'_>) -> Result<Option<TaxProgressDto>, IpcError> {
    let catalog = ctx.require_catalog()?;
    Ok(progress::present_progress(&catalog, today(&ctx)?))
}

/// Runs `job` as a library operation and reports its outcome: `done` on success, if given, and always a failure.
/// Returns whether it succeeded; `false` while the library is busy.
async fn run(
    ctx: &AppState<'_>,
    label: &str,
    done: Option<String>,
    job: impl FnOnce(&dms_application::services::AppServices) -> OperationOutcome + Send + 'static,
) -> Result<bool, IpcError> {
    let finished = wait(ctx.inner(), label, move |services, operation| {
        let outcome = job(services);
        let succeeded = matches!(outcome, OperationOutcome::Done { .. });
        if !succeeded || done.is_some() {
            operation.session().report(messages::of(&outcome, done.as_deref().unwrap_or_default()));
        }
        succeeded
    });
    match finished.await {
        Err(error) if error.code == "busy" => Ok(false),
        other => other,
    }
}

fn change_of(change: TaxItemChangeDto) -> Result<TaxItemChange, IpcError> {
    let item = |id: &str| TaxItemId::new(id).map_err(invalid);
    Ok(match change {
        TaxItemChangeDto::Add { area, label, correspondent } => {
            TaxItemChange::Add { area: area_of(area), label, correspondent }
        }
        TaxItemChangeDto::Rename { item_id, label } => TaxItemChange::Rename { item_id: item(&item_id)?, label },
        TaxItemChangeDto::SetNotNeeded { item_id, not_needed } => {
            TaxItemChange::SetNotNeeded { item_id: item(&item_id)?, not_needed }
        }
        TaxItemChangeDto::LinkDocument { item_id, document_id } => TaxItemChange::LinkDocument {
            item_id: item(&item_id)?,
            document_id: DocumentId::new(document_id).map_err(invalid)?,
        },
    })
}

/// The typed deductible part, such as `684.50` or `684.50 EUR`; blank for none.
fn deductible_of(ctx: &AppContext, raw: Option<&str>) -> Result<Option<Money>, IpcError> {
    let Some(raw) = raw.map(text::trim).filter(|raw| !raw.is_empty()) else {
        return Ok(None);
    };
    let catalog = ctx.require_catalog()?;
    FieldValues::parse(FieldType::Money, raw, catalog.settings().layout.default_currency.as_ref())
        .as_ref()
        .and_then(Money::of_field)
        .map(Some)
        .ok_or_else(|| IpcError::invalid(format!("\"{raw}\" is not an amount")))
}

fn today(ctx: &AppContext) -> Result<chrono::NaiveDate, IpcError> {
    Ok(ctx.require_services()?.library.clock.today())
}

fn invalid(error: impl std::fmt::Display) -> IpcError {
    IpcError::invalid(error.to_string())
}
