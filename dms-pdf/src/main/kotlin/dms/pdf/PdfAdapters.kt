package dms.pdf

import dms.pdf.io.PdfLocations
import dms.pdf.ocr.OcrMyPdf

/** The PDF adapters for one device: the editor on PDFBox and the installed OCR program, if any. */
class PdfAdapters(locations: PdfLocations) {
    val editor: PdfBoxEditor = PdfBoxEditor(locations = locations)
    val ocr: OcrMyPdf = OcrMyPdf(locations = locations)
}
