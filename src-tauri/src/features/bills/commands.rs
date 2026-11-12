//! The commands of the bills screen, of the bill row in the quick look and of the "expected but missing" tasks.
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use std::sync::Arc;

use chrono::NaiveDate;
use dms_application::common::OperationOutcome;
use dms_application::services::AppServices;
use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentId};
use dms_domain::expectation::{ExpectationMode, ExpectationOverride, ExpectedDocument};
use dms_domain::settings::CatalogSettings;
use dms_domain::task::{OpenTask, TaskKey};
use tauri::AppHandle;
use tauri_specta::Event;

use super::dto::{BillDocumentDto, BillPaymentCodeDto, BillsFilterDto, BillsViewDto, ExpectedDocumentDto};
use super::{qr, view};
use crate::commands::AppState;
use crate::context::AppContext;
use crate::dto::IpcError;
use crate::features::document::dto::OpenReviewEvent;
use crate::messages;
use crate::session::reporting::busy;
use crate::session::{Lane, Operation};

/// The "To pay" screen.
#[tauri::command]
#[specta::specta]
pub fn bills_view(ctx: AppState<'_>, filter: BillsFilterDto) -> Result<BillsViewDto, IpcError> {
    Ok(view::bills_view(&ctx.require_catalog()?, today(&ctx), filter))
}

/// Remembers today as the day bills are tracked from, the first time "To pay" opens; older unpaid bills then wait
/// in "Older bills never marked" instead of asking on the dashboard.
#[tauri::command]
#[specta::specta]
pub fn bills_start_tracking(ctx: AppState<'_>) -> Result<(), IpcError> {
    let today = today(&ctx);
    perform(&ctx, "Starting to track bills", move |services, operation| {
        let outcome = services.documents.change_settings.run("Bills tracked from today", |settings| CatalogSettings {
            bills_tracked_since: settings.bills_tracked_since.or(Some(today)),
            ..settings.clone()
        });
        report_failure(operation, &outcome);
    })
}

/// Marks a bill as paid today.
#[tauri::command]
#[specta::specta]
pub fn bills_mark_paid(ctx: AppState<'_>, document_id: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = find_document(&catalog, &document_id)?;
    let (id, path, today) = (document.id.clone(), document.path.file_name().to_owned(), today(&ctx));
    perform(&ctx, &format!("Marking {path} as paid"), move |services, operation| {
        let outcome = services.organize.mark_bill_paid.run(&id, today);
        operation.session().report(messages::of(&outcome, &format!("Marked {path} as paid")));
    })
}

/// Undoes "Mark as paid".
#[tauri::command]
#[specta::specta]
pub fn bills_mark_unpaid(ctx: AppState<'_>, document_id: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = find_document(&catalog, &document_id)?;
    let (id, path) = (document.id.clone(), document.path.file_name().to_owned());
    perform(&ctx, &format!("Marking {path} as not paid"), move |services, operation| {
        let outcome = services.organize.mark_bill_unpaid.run(&id);
        operation.session().report(messages::of(&outcome, &format!("{path} is not paid")));
    })
}

/// "Mark as settled" for the older bills never marked; the window asked for confirmation.
#[tauri::command]
#[specta::specta]
pub fn bills_settle_older(ctx: AppState<'_>) -> Result<(), IpcError> {
    perform(&ctx, "Settling older bills", move |services, operation| {
        let outcome = services.organize.settle_older_bills.run(true);
        operation.session().report(messages::of(&outcome, "Older bills marked as settled"));
    })
}

/// The payment dialog of a bill with its QR code.
#[tauri::command]
#[specta::specta]
pub fn bills_payment_code(ctx: AppState<'_>, document_id: String) -> Result<BillPaymentCodeDto, IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = find_document(&catalog, &document_id)?;
    let (code, payment) = view::payment_code(&catalog, document, today(&ctx));
    Ok(BillPaymentCodeDto { svg: payment.as_ref().and_then(qr::payment_svg), ..code })
}

/// The bill row of the quick look.
#[tauri::command]
#[specta::specta]
pub fn bills_document(ctx: AppState<'_>, document_id: String) -> Result<BillDocumentDto, IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = find_document(&catalog, &document_id)?;
    Ok(view::bill_document(&catalog, document, today(&ctx)))
}

/// Suggests the values of the PDF's e-invoice, without the AI, and opens their review.
#[tauri::command]
#[specta::specta]
pub fn bills_read_e_invoice(ctx: AppState<'_>, app: AppHandle, document_id: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = find_document(&catalog, &document_id)?;
    let (id, name) = (document.id.clone(), document.path.file_name().to_owned());
    perform(&ctx, &format!("Reading the e-invoice of {name}"), move |services, operation| {
        let outcome = services.organize.suggest_from_e_invoice.run(&id);
        let session = operation.session();
        session.report(messages::e_invoice_suggestion(&outcome, &name));
        if matches!(outcome, dms_application::classification::SuggestOutcome::Suggested(_)) {
            session.changed();
            // The window may be gone; the suggestion waits on the dashboard either way.
            let _ = OpenReviewEvent { document_id: id.value().to_owned() }.emit(&app);
        }
    })
}

/// The "expected but missing" documents of the dashboard card.
#[tauri::command]
#[specta::specta]
pub fn bills_expected_missing(ctx: AppState<'_>) -> Result<Vec<ExpectedDocumentDto>, IpcError> {
    let catalog = ctx.require_catalog()?;
    let tasks = ctx.dashboard().map(|dashboard| dashboard.tasks).unwrap_or_default();
    Ok(view::expected_documents(&tasks, &catalog))
}

/// "Requested": hides the task for 14 days on this device.
#[tauri::command]
#[specta::specta]
pub fn bills_expected_requested(ctx: AppState<'_>, task_key: String) -> Result<(), IpcError> {
    let expected = expected_task(&ctx, &task_key)?;
    let key = TaskKey::new(task_key);
    perform(&ctx, "Marking the document as requested", move |services, operation| {
        let session = operation.session();
        match services.organize.snooze_expected_document.run(&key) {
            Ok(()) => {
                session.report(crate::session::UiMessage::info(format!(
                    "Marked as requested from {}; asking again in 14 days",
                    expected.correspondent
                )));
                session.changed();
            }
            Err(error) => session.report(crate::session::UiMessage::error(messages::failure(error.reason))),
        }
    })
}

/// "Stop expecting": never expect documents of this sender and category again.
#[tauri::command]
#[specta::specta]
pub fn bills_expected_stop(ctx: AppState<'_>, task_key: String) -> Result<(), IpcError> {
    let expected = expected_task(&ctx, &task_key)?;
    let label = format!("No longer expecting documents from {}", expected.correspondent);
    perform(&ctx, &label.clone(), move |services, operation| {
        let stop = ExpectationOverride {
            correspondent: expected.correspondent.clone(),
            category_id: expected.category_id.clone(),
            mode: ExpectationMode::Off,
        };
        let outcome = services.documents.change_settings.run(&label, |settings| {
            let mut expectations: Vec<ExpectationOverride> =
                settings.expectations.iter().filter(|known| known.key() != stop.key()).cloned().collect();
            expectations.push(stop.clone());
            CatalogSettings { expectations, ..settings.clone() }
        });
        operation.session().report(messages::of(&outcome, &label));
    })
}

fn expected_task(ctx: &AppContext, task_key: &str) -> Result<ExpectedDocument, IpcError> {
    ctx.dashboard()
        .and_then(|dashboard| {
            dashboard.tasks.into_iter().find_map(|task| match task {
                OpenTask::ExpectedDocumentMissing(expected) if task_key_of(&expected) == task_key => Some(expected),
                _ => None,
            })
        })
        .ok_or_else(|| IpcError::invalid("This task is gone; the dashboard changed in the meantime"))
}

fn task_key_of(expected: &ExpectedDocument) -> String {
    OpenTask::ExpectedDocumentMissing(expected.clone()).key().value().to_owned()
}

fn report_failure(operation: &Operation, outcome: &OperationOutcome) {
    if !matches!(outcome, OperationOutcome::Done { .. }) {
        operation.session().report(messages::of(outcome, ""));
    }
}

/// Runs `job` as the library lane's operation `label`; `busy` names the running one.
fn perform(
    ctx: &AppContext,
    label: &str,
    job: impl FnOnce(&AppServices, &Operation) + Send + 'static,
) -> Result<(), IpcError> {
    let services = Arc::clone(ctx.require_services()?);
    if ctx.session.perform(label, Lane::Library, false, move |operation| {
        job(&services, operation);
        Ok(())
    }) {
        Ok(())
    } else {
        Err(busy(&ctx.session, Lane::Library, label))
    }
}

/// Today in the library's time zone.
fn today(ctx: &AppContext) -> NaiveDate {
    ctx.services().map_or_else(|| chrono::Local::now().date_naive(), |services| services.library.clock.today())
}

/// The catalogued document `raw`, or `invalid_input` when it left the catalog.
fn find_document<'c>(catalog: &'c Catalog, raw: &str) -> Result<&'c Document, IpcError> {
    let id = DocumentId::new(raw).map_err(|error| IpcError::invalid(error.to_string()))?;
    catalog.documents().get(&id).ok_or_else(|| IpcError::invalid("The document is no longer in the catalog"))
}
