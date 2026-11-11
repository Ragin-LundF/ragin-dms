//! Commands of the PDF tools in quick look (Kotlin `PdfActions`, `PageEditorController`).
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use std::sync::Arc;

use dms_application::pdf::{SearchableOutcome, SearchableProgress};
use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentId, DocumentKind};

use super::dto::{MergeCandidateDto, MergeStartDto, PageItemDto, PdfToolsDto};
use super::{backend, view};
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::messages;
use crate::session::reporting::{Reporting, blocking, busy};
use crate::session::{Lane, OperationProgress, UiMessage};

/// Which PDF tools quick look offers for `document_id`.
#[tauri::command]
#[specta::specta]
pub async fn pdf_tools(ctx: AppState<'_>, document_id: String) -> Result<PdfToolsDto, IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = document(&catalog, &document_id)?;
    let name = document.path.file_name().to_owned();
    if document.kind() != DocumentKind::Pdf {
        return Ok(PdfToolsDto { pdf: false, name, ocr_available: false, rebuildable: false });
    }
    let ctx = Arc::clone(&ctx);
    blocking(move || {
        let ocr_available = backend::ocr_available(&ctx)?;
        let rebuildable = backend::rebuild_availability(&ctx, &document.id)?;
        Ok(PdfToolsDto { pdf: true, name, ocr_available, rebuildable })
    })
    .await
}

/// Writes an invisible text layer into the PDF: from the AI's transcription, or with ocrmypdf when `precise`.
#[tauri::command]
#[specta::specta]
pub fn make_searchable(ctx: AppState<'_>, document_id: String, precise: bool) -> Result<(), IpcError> {
    let document = document(&ctx.require_catalog()?, &document_id)?;
    let name = document.path.file_name().to_owned();
    let label = format!("Making {name} searchable");
    let context = Arc::clone(&ctx);
    let started = ctx.session.perform(&label, Lane::Library, false, move |op| {
        let outcome = backend::make_searchable(&context, &document.id, precise).map_err(|error| error.message)?;
        op.session().report(view::searchable_message(&outcome, &name));
        Ok(())
    });
    if started { Ok(()) } else { Err(busy(&ctx.session, Lane::Library, &label)) }
}

/// Makes every scan without text searchable (the dashboard's batch), cancellable.
#[tauri::command]
#[specta::specta]
pub fn make_all_searchable(ctx: AppState<'_>, precise: bool) -> Result<(), IpcError> {
    let label = "Making scans searchable";
    let context = Arc::clone(&ctx);
    let started = ctx.session.perform(label, Lane::Library, true, move |op| {
        let made = std::sync::atomic::AtomicUsize::new(0);
        let progress = Reporting::new(op, |step: &SearchableProgress| {
            op.progress(OperationProgress::new(label, step.done, Some(step.total)));
            if matches!(step.outcome, SearchableOutcome::Made(_)) {
                made.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        });
        backend::make_all_searchable(&context, precise, &progress).map_err(|error| error.message)?;
        if !op.is_cancelled() {
            let made = made.load(std::sync::atomic::Ordering::Relaxed);
            op.session().report(UiMessage::info(format!(
                "Made {made} scans searchable; scans without a transcription were left out"
            )));
        }
        Ok(())
    });
    if started { Ok(()) } else { Err(busy(&ctx.session, Lane::Library, label)) }
}

/// "Merge with…" on `document_id`: it comes first, with the suggested name of the merged file.
#[tauri::command]
#[specta::specta]
pub fn merge_start(ctx: AppState<'_>, document_id: String) -> Result<MergeStartDto, IpcError> {
    Ok(view::merge_start(&document(&ctx.require_catalog()?, &document_id)?))
}

/// Other PDFs to add to the merge, found by `query` in path or title.
#[tauri::command]
#[specta::specta]
pub fn merge_matches(
    ctx: AppState<'_>,
    query: String,
    chosen: Vec<String>,
) -> Result<Vec<MergeCandidateDto>, IpcError> {
    Ok(view::merge_matches(&ctx.require_catalog()?, &query, &chosen))
}

/// Merges the PDFs in this order into `name`; with `trash_sources` they go to the trash afterwards.
#[tauri::command]
#[specta::specta]
pub fn merge_documents(
    ctx: AppState<'_>,
    document_ids: Vec<String>,
    name: String,
    trash_sources: bool,
) -> Result<(), IpcError> {
    let name = dms_domain::text::trim(&name).to_owned();
    if document_ids.len() < 2 || name.is_empty() {
        return Err(IpcError::invalid("Choose at least two PDFs and a name"));
    }
    let ids = document_ids.iter().map(|id| parse_id(id)).collect::<Result<Vec<_>, _>>()?;
    let label = format!("Merging {} PDFs", ids.len());
    let context = Arc::clone(&ctx);
    let started = ctx.session.perform(&label, Lane::Library, false, move |op| {
        let outcome = backend::merge(&context, &ids, &name, trash_sources).map_err(|error| error.message)?;
        op.session().report(messages::of(&outcome, &format!("Merged into {name}")));
        Ok(())
    });
    if started { Ok(()) } else { Err(busy(&ctx.session, Lane::Library, &label)) }
}

/// How many pages the PDF has, for the page editor; 0 when it cannot be read.
#[tauri::command]
#[specta::specta]
pub async fn pdf_page_count(ctx: AppState<'_>, document_id: String) -> Result<u32, IpcError> {
    let id = parse_id(&document_id)?;
    let ctx = Arc::clone(&ctx);
    let count = blocking(move || backend::page_count(&ctx, &id)).await?;
    Ok(u32::try_from(count).unwrap_or(u32::MAX))
}

/// Applies the page editor: writes the turned, removed and moved pages as a new version, then splits where marked.
#[tauri::command]
#[specta::specta]
pub fn edit_pdf_pages(
    ctx: AppState<'_>,
    document_id: String,
    pages: Vec<PageItemDto>,
    trash_original: bool,
) -> Result<(), IpcError> {
    let document = document(&ctx.require_catalog()?, &document_id)?;
    let edit = view::page_edit(&pages).ok_or_else(|| IpcError::invalid("Keep at least one page"))?;
    let name = document.path.file_name().to_owned();
    let label = format!("Editing the pages of {name}");
    let context = Arc::clone(&ctx);
    let started = ctx.session.perform(&label, Lane::Library, false, move |op| {
        if let Some(changes) = &edit.changes {
            let outcome = backend::edit_pages(&context, &document.id, changes).map_err(|error| error.message)?;
            if !matches!(outcome, dms_application::common::OperationOutcome::Done { .. }) {
                op.session().report(messages::of(&outcome, ""));
                return Ok(());
            }
        }
        if edit.ranges.len() < 2 {
            op.session().report(UiMessage::info(format!("Saved the pages of {name}")));
            return Ok(());
        }
        let outcome =
            backend::split(&context, &document.id, &edit.ranges, trash_original).map_err(|error| error.message)?;
        let done = format!("Split {name} into {} documents", edit.ranges.len());
        op.session().report(messages::of(&outcome, &done));
        Ok(())
    });
    if started { Ok(()) } else { Err(busy(&ctx.session, Lane::Library, &label)) }
}

pub(super) fn parse_id(raw: &str) -> Result<DocumentId, IpcError> {
    DocumentId::new(raw).map_err(|error| IpcError::invalid(error.to_string()))
}

pub(super) fn document(catalog: &Catalog, raw_id: &str) -> Result<Document, IpcError> {
    catalog
        .documents()
        .get(&parse_id(raw_id)?)
        .cloned()
        .ok_or_else(|| IpcError::invalid("The document or file no longer exists"))
}
