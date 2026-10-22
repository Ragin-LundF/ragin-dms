package dms.pdf.meta

import dms.application.port.pdf.model.PdfMetadataModel
import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.pdf.io.PdfFiles
import org.apache.pdfbox.pdmodel.PDDocument
import org.apache.pdfbox.pdmodel.common.PDMetadata
import org.apache.xmpbox.XMPMetadata
import org.apache.xmpbox.xml.XmpSerializer
import java.io.ByteArrayOutputStream
import java.nio.file.Path
import java.time.LocalDate
import java.time.ZoneOffset
import java.util.Calendar
import java.util.GregorianCalendar
import java.util.TimeZone

/**
 * The document information and the XMP metadata of a PDF: title, author (correspondent), subject
 * (category), keywords (tags) and creation date (the document date), so other applications find them.
 */
object PdfInfo {

    private const val KEYWORD_SEPARATOR = ", "

    fun read(file: Path): PdfMetadataModel {
        PdfFiles.load(file = file).use { pdf ->
            val info = pdf.documentInformation
            return PdfMetadataModel(
                title = info.title?.takeIf { it.isNotBlank() },
                author = info.author?.takeIf { it.isNotBlank() },
                subject = info.subject?.takeIf { it.isNotBlank() },
                keywords = info.keywords?.split(',')?.map { it.trim() }?.filter { it.isNotEmpty() }.orEmpty(),
                date = info.creationDate?.let { LocalDate.ofInstant(it.toInstant(), ZoneOffset.UTC) },
            )
        }
    }

    fun write(source: Path, metadata: PdfMetadataModel, target: Path): StorageResult<FileFactsModel> {
        PdfFiles.load(file = source).use { pdf ->
            val info = pdf.documentInformation
            info.title = metadata.title
            info.author = metadata.author
            info.subject = metadata.subject
            info.keywords = metadata.keywords.joinToString(separator = KEYWORD_SEPARATOR).ifEmpty { null }
            info.creationDate = metadata.date?.let { calendarOf(date = it) }
            pdf.documentCatalog.metadata = xmpOf(pdf = pdf, metadata = metadata)
            return PdfFiles.save(document = pdf, target = target)
        }
    }

    private fun xmpOf(pdf: PDDocument, metadata: PdfMetadataModel): PDMetadata {
        val xmp = XMPMetadata.createXMPMetadata()
        val dublinCore = xmp.createAndAddDublinCoreSchema()
        metadata.title?.let { dublinCore.title = it }
        metadata.author?.let { dublinCore.addCreator(it) }
        metadata.subject?.let { dublinCore.addDescription(null, it) }
        metadata.keywords.forEach { dublinCore.addSubject(it) }
        metadata.date?.let { dublinCore.addDate(calendarOf(date = it)) }
        val pdfSchema = xmp.createAndAddAdobePDFSchema()
        pdfSchema.keywords = metadata.keywords.joinToString(separator = KEYWORD_SEPARATOR)
        val bytes = ByteArrayOutputStream().use { output ->
            XmpSerializer().serialize(xmp, output, true)
            output.toByteArray()
        }
        val stream = PDMetadata(pdf)
        stream.importXMPMetadata(bytes)
        return stream
    }

    private fun calendarOf(date: LocalDate): Calendar {
        val calendar = GregorianCalendar(TimeZone.getTimeZone("UTC"))
        calendar.clear()
        calendar.set(date.year, date.monthValue - 1, date.dayOfMonth)
        return calendar
    }
}
