package dms.application.port.pdf.model

import java.time.LocalDate

/**
 * What a PDF carries about itself in its document information and XMP: title, author (the
 * correspondent), subject (the category), keywords (the tags) and the document date.
 */
data class PdfMetadataModel(
    val title: String? = null,
    val author: String? = null,
    val subject: String? = null,
    val keywords: List<String> = emptyList(),
    val date: LocalDate? = null,
)
