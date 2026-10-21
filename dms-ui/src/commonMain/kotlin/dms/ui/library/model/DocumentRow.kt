package dms.ui.library.model

import dms.domain.document.Document

/** One line of the document list. */
data class DocumentRow(
    val document: Document,
    val categoryName: String?,
)
