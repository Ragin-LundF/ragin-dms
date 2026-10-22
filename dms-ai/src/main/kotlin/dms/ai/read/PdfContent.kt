package dms.ai.read

import dms.ai.prompt.PromptTexts
import dms.application.port.ai.model.DocumentContent
import dms.application.port.ai.model.PageImageModel
import org.apache.pdfbox.Loader
import org.apache.pdfbox.io.RandomAccessReadBufferedFile
import org.apache.pdfbox.rendering.ImageType
import org.apache.pdfbox.rendering.PDFRenderer
import org.apache.pdfbox.text.PDFTextStripper
import java.nio.file.Path

/**
 * A PDF as text from its text layer, or, for a scan without one (or with [preferVision]), its first
 * pages rendered for a vision model.
 */
object PdfContent {

    private const val TEXT_PAGES = 10
    private const val VISION_PAGES = 2
    private const val DPI = 150f

    /** An A4 page at this resolution is about 400 pixels wide, enough for any thumbnail. */
    private const val THUMBNAIL_DPI = 48f

    /** Fewer characters than this is a scan with at most a page number or a stamp as text. */
    private const val MIN_TEXT = 80

    /** The whole text layer, or an empty string for a scan. */
    fun text(file: Path): String {
        return Loader.loadPDF(RandomAccessReadBufferedFile(file.toString())).use { pdf ->
            PDFTextStripper().getText(pdf).trim()
        }
    }

    /** The first page, at most [maxSide] pixels on its long side; `null` for a PDF without pages. */
    fun thumbnail(file: Path, maxSide: Int): PageImageModel? {
        return Loader.loadPDF(RandomAccessReadBufferedFile(file.toString())).use { pdf ->
            if (pdf.numberOfPages == 0) {
                null
            } else {
                val page = PDFRenderer(pdf).renderImageWithDPI(0, THUMBNAIL_DPI, ImageType.RGB)
                VisionImages.of(image = page, maxSide = maxSide)
            }
        }
    }

    fun read(file: Path, preferVision: Boolean): DocumentContent {
        return Loader.loadPDF(RandomAccessReadBufferedFile(file.toString())).use { pdf ->
            val text = if (preferVision) {
                ""
            } else {
                val stripper = PDFTextStripper().apply {
                    startPage = 1
                    endPage = minOf(TEXT_PAGES, pdf.numberOfPages)
                }
                stripper.getText(pdf).trim()
            }
            if (text.length >= MIN_TEXT) {
                DocumentContent.Text(text = text.take(n = PromptTexts.MAX_TEXT_CHARACTERS))
            } else if (pdf.numberOfPages == 0) {
                DocumentContent.NotReadable(reason = "The PDF has no pages")
            } else {
                val renderer = PDFRenderer(pdf)
                val pages = (0 until minOf(VISION_PAGES, pdf.numberOfPages)).map { index ->
                    VisionImages.of(image = renderer.renderImageWithDPI(index, DPI, ImageType.RGB))
                }
                DocumentContent.PageImages(pages = pages)
            }
        }
    }
}
