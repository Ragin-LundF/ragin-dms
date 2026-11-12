//! Commands of the split dialog: the proposal runs in the AI lane, the split itself in the library lane.
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use std::sync::{Arc, mpsc};

use dms_application::split::{ApplySplitInput, ApplySplitOutcome, SplitProgress};
use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentId, DocumentKind};

use super::dto::{SplitApplyInputDto, SplitProposalDto};
use super::view;
use crate::commands::AppState;
use crate::context::AppContext;
use crate::dto::IpcError;
use crate::features::document::view::texts::suggest_message;
use crate::messages;
use crate::session::reporting::{Reporting, blocking, busy};
use crate::session::{Lane, OperationProgress};

/// A stack needs this many pages before the inbox offers to split it.
const INBOX_STACK_PAGES: usize = 3;

/// The AI's proposal where each document of the PDF starts. Reading a scan page by page reports progress and can
/// be cancelled.
#[tauri::command]
#[specta::specta]
pub async fn split_propose(ctx: AppState<'_>, document_id: String) -> Result<SplitProposalDto, IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = document(&catalog, &document_id)?;
    let services = Arc::clone(ctx.require_services()?);
    let label = format!("Finding the documents in {}", document.path.file_name());
    let (sender, receiver) = mpsc::channel();
    let id = document.id.clone();
    let step = label.clone();
    let started = ctx.session.perform(&label, Lane::Ai, true, move |op| {
        let progress = Reporting::new(op, |page: &SplitProgress| {
            op.progress(OperationProgress::new(&step, page.done, Some(page.total)).with_detail("Reading the pages"));
        });
        // The window may have stopped waiting; the proposal is then not needed.
        let _ = sender.send(services.pdf.propose_split.run(&id, &progress));
        Ok(())
    });
    if !started {
        return Err(busy(&ctx.session, Lane::Ai, &label));
    }
    let outcome = blocking(move || receiver.recv().map_err(IpcError::internal)).await?;
    view::proposal_dto(outcome, &document, &catalog)
}

/// Splits the PDF at the confirmed cuts; the parts the AI said nothing about are then classified in the AI lane.
#[tauri::command]
#[specta::specta]
pub fn split_apply(ctx: AppState<'_>, input: SplitApplyInputDto) -> Result<(), IpcError> {
    let document = document(&ctx.require_catalog()?, &input.document_id)?;
    let name = document.path.file_name().to_owned();
    let split = ApplySplitInput {
        document_id: document.id,
        parts: input.parts.iter().map(view::part_of).collect(),
        model: input.model,
        trash_original: input.trash_original,
        confirmed: true,
    };
    let label = format!("Splitting {name}");
    let context = Arc::clone(&ctx);
    let started = ctx.session.perform(&label, Lane::Library, false, move |op| {
        let services = context.require_services().map_err(|error| error.message)?;
        let outcome = services.pdf.apply_split.run(&split);
        op.session().report(messages::split_applied(&outcome, &name));
        op.session().changed();
        if let ApplySplitOutcome::Split { without_suggestion, .. } = outcome {
            classify(&context, without_suggestion);
        }
        Ok(())
    });
    if started { Ok(()) } else { Err(busy(&ctx.session, Lane::Library, &label)) }
}

/// The inbox PDFs of at least three pages, while the AI is on and the inbox proposes splits; empty otherwise.
#[tauri::command]
#[specta::specta]
pub async fn split_inbox_candidates(ctx: AppState<'_>) -> Result<Vec<String>, IpcError> {
    let Some(catalog) = ctx.catalog() else { return Ok(Vec::new()) };
    let inbox = catalog.settings().layout.inbox.clone();
    if !ctx.session.state().ai_enabled || !inbox.enabled || !inbox.propose_splits {
        return Ok(Vec::new());
    }
    let services = Arc::clone(ctx.require_services()?);
    blocking(move || {
        let library = services.library.store.current().ok_or_else(IpcError::not_open)?;
        let folder = library.folders.primary();
        // shortcut: opens every inbox PDF on each call; cache page counts per content hash if inboxes grow large.
        Ok(library
            .catalog
            .documents()
            .values()
            .filter(|document| inbox.contains(&document.path) && document.kind() == DocumentKind::Pdf)
            .filter(|document| {
                services.pdf.editor.page_count(folder, &document.path).is_ok_and(|pages| pages >= INBOX_STACK_PAGES)
            })
            .map(|document| document.id.value().to_owned())
            .collect())
    })
    .await
}

/// Lets the AI suggest a classification for each part, as "Analyzing the split parts" in the AI lane; skipped while
/// the AI lane is busy, because the parts can be analyzed later from the inbox or quick look.
fn classify(ctx: &Arc<AppContext>, documents: Vec<DocumentId>) {
    if documents.is_empty() || !ctx.session.state().ai_enabled {
        return;
    }
    let context = Arc::clone(ctx);
    let _started = ctx.session.perform("Analyzing the split parts", Lane::Ai, true, move |op| {
        let services = context.require_services().map_err(|error| error.message)?;
        for (done, id) in documents.iter().enumerate() {
            if op.is_cancelled() {
                break;
            }
            op.progress(OperationProgress::new("Analyzing the split parts", done, Some(documents.len())));
            let name = context
                .catalog()
                .and_then(|catalog| catalog.documents().get(id).map(|document| document.path.file_name().to_owned()))
                .unwrap_or_default();
            let outcome = services.ai.suggest_classification.run(id);
            op.session().report(suggest_message(&outcome, &name));
        }
        op.session().changed();
        Ok(())
    });
}

fn document(catalog: &Catalog, raw_id: &str) -> Result<Document, IpcError> {
    let id = DocumentId::new(raw_id).map_err(|error| IpcError::invalid(error.to_string()))?;
    catalog.documents().get(&id).cloned().ok_or_else(|| IpcError::invalid("The document or file no longer exists"))
}
