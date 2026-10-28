package dms.pdf

import dms.pdf.io.PdfLocations
import dms.pdf.ocr.OcrMyPdf
import dms.pdf.rebuild.PdfBoxRebuilder
import java.nio.file.Path

/**
 * The PDF adapters for one device: the editor on PDFBox, the installed OCR program, if any, and the
 * rebuild, whose results wait in [rebuildDirectory] in the app's own data.
 */
class PdfAdapters(locations: PdfLocations, rebuildDirectory: Path) {
    val editor: PdfBoxEditor = PdfBoxEditor(locations = locations)
    val ocr: OcrMyPdf = OcrMyPdf(locations = locations)
    val rebuilder: PdfBoxRebuilder = PdfBoxRebuilder(locations = locations, workDirectory = rebuildDirectory)
}
