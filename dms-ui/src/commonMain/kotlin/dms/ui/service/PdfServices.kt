package dms.ui.service

import dms.application.pdf.EditPages
import dms.application.pdf.EmbedPdfMetadata
import dms.application.pdf.MakeAllSearchable
import dms.application.pdf.MakeSearchable
import dms.application.pdf.MergeDocuments
import dms.application.pdf.ScansWithoutText
import dms.application.pdf.SplitDocument
import dms.application.pdf.rebuild.KeepRebuild
import dms.application.pdf.rebuild.RebuildAvailability
import dms.application.pdf.rebuild.RebuildPdf
import dms.application.port.pdf.OcrTool
import dms.application.port.pdf.PdfEditor
import dms.application.port.pdf.PdfRebuilder

/**
 * The PDF tools: pages, split and merge, searchable scans, embedded metadata and the conversion of scans
 * to real PDFs; [editor] and [rebuilder] also draw page pictures.
 */
class PdfServices(
    val editPages: EditPages,
    val splitDocument: SplitDocument,
    val mergeDocuments: MergeDocuments,
    val makeSearchable: MakeSearchable,
    val makeAllSearchable: MakeAllSearchable,
    val scansWithoutText: ScansWithoutText,
    val embedPdfMetadata: EmbedPdfMetadata,
    val editor: PdfEditor,
    val ocr: OcrTool,
    val rebuildAvailability: RebuildAvailability,
    val rebuildPdf: RebuildPdf,
    val keepRebuild: KeepRebuild,
    val rebuilder: PdfRebuilder,
)
