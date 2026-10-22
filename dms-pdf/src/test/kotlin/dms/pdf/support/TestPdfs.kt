package dms.pdf.support

import dms.domain.library.FolderRole
import dms.domain.library.model.FolderLocationModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import dms.pdf.io.PdfLocations
import org.apache.pdfbox.pdmodel.PDDocument
import org.apache.pdfbox.pdmodel.PDPage
import org.apache.pdfbox.pdmodel.PDPageContentStream
import org.apache.pdfbox.pdmodel.common.PDRectangle
import org.apache.pdfbox.pdmodel.font.PDType1Font
import org.apache.pdfbox.pdmodel.font.Standard14Fonts
import java.nio.file.Files
import java.nio.file.Path

/** Synthetic PDFs and a library folder on a temporary directory; never real documents. */
object TestPdfs {

    fun folder(root: Path): LibraryFolderModel {
        return LibraryFolderModel(
            id = LibraryFolderIdModel(value = "primary"),
            displayName = "primary",
            location = FolderLocationModel(value = root.toString()),
            role = FolderRole.PRIMARY,
        )
    }

    /** Plain path resolution for tests; the app passes the storage adapter's NFC-aware one. */
    fun locations(root: Path): PdfLocations {
        return PdfLocations(
            existing = { _, path -> root.resolve(path.value).takeIf { Files.exists(it) } },
            target = { _, path -> root.resolve(path.value) },
        )
    }

    /** A PDF with one page per text; a `null` text makes a page without any text, like a scan. */
    fun write(file: Path, pages: List<String?>, widths: List<Float> = emptyList()) {
        Files.createDirectories(file.parent)
        PDDocument().use { pdf ->
            pages.forEachIndexed { index, text ->
                val width = widths.getOrElse(index = index) { PDRectangle.A4.width }
                val page = PDPage(PDRectangle(width, PDRectangle.A4.height))
                pdf.addPage(page)
                if (text != null) {
                    PDPageContentStream(pdf, page).use { stream ->
                        stream.beginText()
                        stream.setFont(PDType1Font(Standard14Fonts.FontName.HELVETICA), 12f)
                        stream.newLineAtOffset(50f, 700f)
                        stream.showText(text)
                        stream.endText()
                    }
                }
            }
            pdf.save(file.toFile())
        }
    }

    fun path(raw: String): RelativePathModel {
        return RelativePathModel.of(raw = raw)
    }
}
