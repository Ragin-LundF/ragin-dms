//! The use cases and ports the PDF tools call (`AppServices`).

use std::ops::RangeInclusive;
use std::sync::OnceLock;

use dms_application::common::{OperationOutcome, Progress};
use dms_application::pdf::rebuild::{RebuildChoice, RebuildOutcome};
use dms_application::pdf::{SearchableOutcome, SearchableProgress};
use dms_application::port::pdf::{PageChange, RebuiltFile};
use dms_domain::document::DocumentId;

use crate::context::AppContext;
use crate::dto::IpcError;

/// Whether ocrmypdf is installed, probed once per app start, as Kotlin's `PdfActions` did.
pub fn ocr_available(ctx: &AppContext) -> Result<bool, IpcError> {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    let services = ctx.require_services()?;
    Ok(*AVAILABLE.get_or_init(|| services.pdf.ocr.available()))
}

pub fn rebuild_availability(ctx: &AppContext, document_id: &DocumentId) -> Result<bool, IpcError> {
    Ok(ctx.require_services()?.pdf.rebuild_availability.run(document_id))
}

pub fn make_searchable(
    ctx: &AppContext,
    document_id: &DocumentId,
    precise: bool,
) -> Result<SearchableOutcome, IpcError> {
    Ok(ctx.require_services()?.pdf.make_searchable.run(document_id, precise))
}

pub fn make_all_searchable(
    ctx: &AppContext,
    precise: bool,
    progress: &dyn Progress<SearchableProgress>,
) -> Result<Option<SearchableProgress>, IpcError> {
    Ok(ctx.require_services()?.pdf.make_all_searchable.run(precise, progress))
}

pub fn merge(
    ctx: &AppContext,
    document_ids: &[DocumentId],
    file_name: &str,
    trash_sources: bool,
) -> Result<OperationOutcome, IpcError> {
    Ok(ctx.require_services()?.pdf.merge_documents.run(document_ids, file_name, trash_sources))
}

/// Pages of the document's file in the primary folder; 0 when it cannot be read (as Kotlin did).
pub fn page_count(ctx: &AppContext, document_id: &DocumentId) -> Result<usize, IpcError> {
    let services = ctx.require_services()?;
    let library = services.library.store.current().ok_or_else(IpcError::not_open)?;
    let document = library
        .catalog
        .documents()
        .get(document_id)
        .ok_or_else(|| IpcError::invalid("The document or file no longer exists"))?;
    Ok(services.pdf.editor.page_count(library.folders.primary(), &document.path).unwrap_or(0))
}

pub fn edit_pages(
    ctx: &AppContext,
    document_id: &DocumentId,
    pages: &[PageChange],
) -> Result<OperationOutcome, IpcError> {
    Ok(ctx.require_services()?.pdf.edit_pages.run(document_id, pages))
}

pub fn split(
    ctx: &AppContext,
    document_id: &DocumentId,
    ranges: &[RangeInclusive<usize>],
    trash_original: bool,
) -> Result<OperationOutcome, IpcError> {
    Ok(ctx.require_services()?.pdf.split_document.run(document_id, ranges, trash_original))
}

pub fn rebuild(
    ctx: &AppContext,
    document_id: &DocumentId,
    progress: &dyn Progress<(usize, usize)>,
) -> Result<RebuildOutcome, IpcError> {
    Ok(ctx.require_services()?.pdf.rebuild_pdf.run(document_id, progress))
}

pub fn keep_rebuild(
    ctx: &AppContext,
    document_id: &DocumentId,
    rebuilt: &RebuiltFile,
    choice: RebuildChoice,
) -> Result<OperationOutcome, IpcError> {
    Ok(ctx.require_services()?.pdf.keep_rebuild.run(document_id, rebuilt, choice))
}
