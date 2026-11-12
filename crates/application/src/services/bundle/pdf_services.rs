use std::sync::Arc;

use crate::pdf::rebuild::{KeepRebuild, RebuildAvailability, RebuildPdf};
use crate::pdf::{
    EditPages, EmbedPdfMetadata, MakeAllSearchable, MakeSearchable, MergeDocuments, ScansWithoutText, SplitDocument,
};
use crate::port::pdf::{OcrTool, PdfEditor, PdfRebuilder};
use crate::split::{ApplySplit, ProposeSplit};

/// The PDF tools: pages, split and merge, the AI's split of scanned stacks, searchable scans, embedded metadata and the conversion of scans to real
/// PDFs; `editor` and `rebuilder` also draw page pictures.
pub struct PdfServices {
    pub edit_pages: Arc<EditPages>,
    pub split_document: Arc<SplitDocument>,
    pub propose_split: Arc<ProposeSplit>,
    pub apply_split: Arc<ApplySplit>,
    pub merge_documents: Arc<MergeDocuments>,
    pub make_searchable: Arc<MakeSearchable>,
    pub make_all_searchable: Arc<MakeAllSearchable>,
    pub scans_without_text: Arc<ScansWithoutText>,
    pub embed_pdf_metadata: Arc<EmbedPdfMetadata>,
    pub editor: Arc<dyn PdfEditor>,
    pub ocr: Arc<dyn OcrTool>,
    pub rebuild_availability: Arc<RebuildAvailability>,
    pub rebuild_pdf: Arc<RebuildPdf>,
    pub keep_rebuild: Arc<KeepRebuild>,
    pub rebuilder: Arc<dyn PdfRebuilder>,
}
