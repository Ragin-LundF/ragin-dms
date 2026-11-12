//! The commands of the contracts screen, the quick look's contract row and the dashboard's contract card. Changes
//! run in the library lane, the AI's reading in the AI lane.
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use std::sync::Arc;

use dms_application::common::OperationOutcome;
use dms_application::contract::{AttachDocumentsInput, ContractDraft, ExtractContractTermsOutcome};
use dms_application::services::AppServices;
use dms_domain::catalog::Catalog;
use dms_domain::contract::{Contract, ContractId};
use dms_domain::document::DocumentId;
use dms_domain::field::CurrencyCode;
use dms_domain::parse_iso_date;

use super::dto::{
    ContractEditorDto, ContractPricePointDto, ContractTermsInputDto, ContractTermsReviewDto, ContractsOfDocumentDto,
    ContractsStatusDto, ContractsViewDto,
};
use super::{editor, view};
use crate::commands::AppState;
use crate::context::AppContext;
use crate::dto::IpcError;
use crate::features::letters::commands::in_lane;
use crate::messages;
use crate::session::Lane;

#[tauri::command]
#[specta::specta]
pub fn contracts_view(ctx: AppState<'_>) -> Result<ContractsViewDto, IpcError> {
    Ok(view::contracts_view(&ctx.require_catalog()?, today(&ctx)))
}

/// The dashboard's card "Contracts per month"; `None` while no running contract has a price.
#[tauri::command]
#[specta::specta]
pub fn contracts_status(ctx: AppState<'_>) -> Result<Option<ContractsStatusDto>, IpcError> {
    Ok(view::contracts_status(&ctx.require_catalog()?, today(&ctx)))
}

#[tauri::command]
#[specta::specta]
pub fn contracts_of_document(ctx: AppState<'_>, document_id: String) -> Result<ContractsOfDocumentDto, IpcError> {
    let id = DocumentId::new(document_id).map_err(|error| IpcError::invalid(error.to_string()))?;
    Ok(view::contracts_of_document(&ctx.require_catalog()?, &id, today(&ctx)))
}

/// The editor's form of a new contract for `correspondent` with `document_ids` (a proposal, the quick look).
#[tauri::command]
#[specta::specta]
pub fn contracts_new_form(correspondent: String, document_ids: Vec<String>) -> ContractEditorDto {
    editor::new_form(&correspondent, document_ids)
}

/// Records a new contract or saves the edited one; answers its id.
#[tauri::command]
#[specta::specta]
pub async fn contracts_save(ctx: AppState<'_>, form: ContractEditorDto) -> Result<String, IpcError> {
    let catalog = ctx.require_catalog()?;
    let draft = editor::draft_of(&form, currency(&catalog)).map_err(IpcError::invalid)?;
    let done = format!("Saved the contract {}", draft.name);
    let ctx = Arc::clone(&ctx);
    let saved = in_lane(&ctx, "Saving the contract", Lane::Library, move |services| {
        services.organize.save_contract.run(&draft)
    })
    .await?;
    reported(&ctx, &saved.outcome, &done)?;
    Ok(saved.contract_id.to_string())
}

/// Ends the contract on `ended_on` (`YYYY-MM-DD`), or lets it run again when empty.
#[tauri::command]
#[specta::specta]
pub async fn contracts_end(ctx: AppState<'_>, contract_id: String, ended_on: String) -> Result<(), IpcError> {
    let id = contract_id_of(&contract_id)?;
    let ended_on = match ended_on.trim() {
        "" => None,
        raw => Some(parse_iso_date(raw).ok_or_else(|| IpcError::invalid("The end is not a date (YYYY-MM-DD)"))?),
    };
    let done = if ended_on.is_some() { "Ended the contract" } else { "The contract runs again" };
    run_change(&ctx, "Ending the contract", done, move |services| services.organize.end_contract.run(&id, ended_on))
        .await
}

/// Removes the contract after the window asked; its documents stay.
#[tauri::command]
#[specta::specta]
pub async fn contracts_delete(ctx: AppState<'_>, contract_id: String, confirmed: bool) -> Result<(), IpcError> {
    let id = contract_id_of(&contract_id)?;
    run_change(&ctx, "Deleting the contract", "Deleted the contract; its documents stay", move |services| {
        services.organize.delete_contract.run(&id, confirmed)
    })
    .await
}

/// Attaches documents to the contract, or with `detach` takes them off it.
#[tauri::command]
#[specta::specta]
pub async fn contracts_attach(
    ctx: AppState<'_>,
    contract_id: String,
    document_ids: Vec<String>,
    detach: bool,
) -> Result<(), IpcError> {
    let input = AttachDocumentsInput {
        contract_id: contract_id_of(&contract_id)?,
        document_ids: document_ids
            .iter()
            .map(|id| DocumentId::new(id).map_err(|error| IpcError::invalid(error.to_string())))
            .collect::<Result<_, _>>()?,
        detach,
    };
    let done = if detach { "Taken off the contract" } else { "Added to the contract" };
    run_change(&ctx, "Changing the contract's documents", done, move |services| {
        services.organize.attach_documents_to_contract.run(&input)
    })
    .await
}

/// Lets the AI read prices and terms from the contract's documents, for the review.
#[tauri::command]
#[specta::specta]
pub async fn contracts_extract_terms(
    ctx: AppState<'_>,
    contract_id: String,
) -> Result<ContractTermsReviewDto, IpcError> {
    let id = contract_id_of(&contract_id)?;
    let ctx = Arc::clone(&ctx);
    let lookup = id.clone();
    let outcome = in_lane(&ctx, "Reading the contract's terms", Lane::Ai, move |services| {
        services.organize.extract_contract_terms.run(&lookup)
    })
    .await?;
    let ExtractContractTermsOutcome::Suggested(suggestion) = &outcome else {
        return Err(IpcError::invalid(messages::contract_terms_problem(&outcome).unwrap_or_default()));
    };
    let catalog = ctx.require_catalog()?;
    let contract = find(&catalog, &id)?;
    Ok(view::terms_review(suggestion, contract, &catalog))
}

/// Adds one price point the user accepted from the AI's reading.
#[tauri::command]
#[specta::specta]
pub async fn contracts_accept_price(
    ctx: AppState<'_>,
    contract_id: String,
    point: ContractPricePointDto,
) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let contract = find(&catalog, &contract_id_of(&contract_id)?)?;
    let point = editor::point_of(&point, currency(&catalog)).map_err(IpcError::invalid)?;
    let draft = ContractDraft { added_prices: vec![point], ..ContractDraft::of(contract) };
    save_draft(&ctx, draft, "Added the price").await
}

/// Takes over the terms the user accepted from the AI's reading.
#[tauri::command]
#[specta::specta]
pub async fn contracts_accept_terms(
    ctx: AppState<'_>,
    contract_id: String,
    terms: ContractTermsInputDto,
) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let contract = find(&catalog, &contract_id_of(&contract_id)?)?;
    let terms = editor::terms_with(&contract.terms, &terms).map_err(IpcError::invalid)?;
    save_draft(&ctx, ContractDraft { terms, ..ContractDraft::of(contract) }, "Took over the terms").await
}

async fn save_draft(ctx: &Arc<AppContext>, draft: ContractDraft, done: &str) -> Result<(), IpcError> {
    run_change(ctx, "Saving the contract", done, move |services| services.organize.save_contract.run(&draft).outcome)
        .await
}

/// Runs a change in the library lane and reports `done`, or answers why nothing changed.
async fn run_change(
    ctx: &Arc<AppContext>,
    label: &str,
    done: &str,
    change: impl FnOnce(&AppServices) -> OperationOutcome + Send + 'static,
) -> Result<(), IpcError> {
    let outcome = in_lane(ctx, label, Lane::Library, change).await?;
    reported(ctx, &outcome, done)
}

fn reported(ctx: &AppContext, outcome: &OperationOutcome, done: &str) -> Result<(), IpcError> {
    let message = messages::of(outcome, done);
    if matches!(outcome, OperationOutcome::Done { .. }) {
        ctx.session.report(message);
        Ok(())
    } else {
        Err(IpcError::invalid(message.text))
    }
}

fn contract_id_of(raw: &str) -> Result<ContractId, IpcError> {
    ContractId::new(raw).map_err(|error| IpcError::invalid(error.to_string()))
}

fn find<'c>(catalog: &'c Catalog, id: &ContractId) -> Result<&'c Contract, IpcError> {
    catalog
        .contracts()
        .get(id)
        .ok_or_else(|| IpcError::invalid(messages::rejection(dms_domain::catalog::CatalogRejection::ContractNotFound)))
}

fn currency(catalog: &Catalog) -> Option<&CurrencyCode> {
    catalog.settings().layout.default_currency.as_ref()
}

/// Today in the library's time zone.
fn today(ctx: &AppContext) -> chrono::NaiveDate {
    ctx.services().map_or_else(|| chrono::Local::now().date_naive(), |services| services.library.clock.today())
}
