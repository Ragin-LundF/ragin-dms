package dms.pdf.text

import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.pdf.io.PdfFiles
import org.apache.pdfbox.pdmodel.PDDocument
import org.apache.pdfbox.pdmodel.PDPageContentStream
import org.apache.pdfbox.pdmodel.font.PDType1Font
import org.apache.pdfbox.pdmodel.font.Standard14Fonts
import org.apache.pdfbox.pdmodel.graphics.state.RenderingMode
import org.apache.pdfbox.text.PDFTextStripper
import java.nio.file.Path

/**
 * The text layer of a PDF: whether it has a usable one, and adding an invisible one (render mode
 * "neither", as OCR programs do) from a transcription, so a scan can be searched and its text copied.
 * Without the positions of the words, the text is laid out line by line from the top of the page.
 */
object TextLayer {

    /** A scan may carry a page number or a stamp as text; less than this per page is no text layer. */
    private const val MIN_CHARACTERS_PER_PAGE = 20
    private const val FONT_SIZE = 8f
    private const val LEADING = 10f
    private const val MARGIN = 36f

    fun hasText(file: Path): Boolean {
        PdfFiles.load(file = file).use { pdf ->
            if (pdf.numberOfPages == 0) {
                return false
            }
            val text = PDFTextStripper().getText(pdf).filterNot { it.isWhitespace() }
            return text.length >= MIN_CHARACTERS_PER_PAGE * pdf.numberOfPages
        }
    }

    /** A copy of [source] with one text per page as invisible text; extra texts go on the last page. */
    fun add(source: Path, pages: List<String>, target: Path): StorageResult<FileFactsModel> {
        PdfFiles.load(file = source).use { pdf ->
            val font = PDType1Font(Standard14Fonts.FontName.HELVETICA)
            val texts = spread(pages = pages, count = pdf.numberOfPages)
            pdf.pages.forEachIndexed { index, page ->
                val text = texts.getOrNull(index = index).orEmpty()
                if (text.isNotBlank()) {
                    write(pdf = pdf, page = page, font = font, text = text)
                }
            }
            return PdfFiles.save(document = pdf, target = target)
        }
    }

    private fun spread(pages: List<String>, count: Int): List<String> {
        if (pages.size <= count) {
            return pages
        }
        return pages.take(n = count - 1) + pages.drop(n = count - 1).joinToString(separator = "\n")
    }

    private fun write(pdf: PDDocument, page: org.apache.pdfbox.pdmodel.PDPage, font: PDType1Font, text: String) {
        PDPageContentStream(pdf, page, PDPageContentStream.AppendMode.APPEND, true, true).use { stream ->
            stream.beginText()
            stream.setRenderingMode(RenderingMode.NEITHER)
            stream.setFont(font, FONT_SIZE)
            stream.setLeading(LEADING)
            stream.newLineAtOffset(MARGIN, page.mediaBox.height - MARGIN)
            text.lines().forEach { line ->
                stream.showText(encodable(font = font, text = line))
                stream.newLine()
            }
            stream.endText()
        }
    }

    /** The standard font cannot show every character; those it cannot are left out, the rest stays searchable. */
    private fun encodable(font: PDType1Font, text: String): String {
        return text.filter { character ->
            !character.isISOControl() && runCatching { font.encode(character.toString()) }.isSuccess
        }
    }
}
