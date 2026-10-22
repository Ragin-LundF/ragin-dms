package dms.ui.service

import dms.application.pdf.EditPages
import dms.application.pdf.EmbedPdfMetadata
import dms.application.pdf.MakeAllSearchable
import dms.application.pdf.MakeSearchable
import dms.application.pdf.MergeDocuments
import dms.application.pdf.ScansWithoutText
import dms.application.pdf.SplitDocument
import dms.application.port.pdf.OcrTool
import dms.application.port.pdf.PdfEditor

/** The PDF tools: pages, split and merge, searchable scans and embedded metadata; [editor] also draws page pictures. */
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
)
