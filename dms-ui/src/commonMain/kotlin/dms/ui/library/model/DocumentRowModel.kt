package dms.ui.library.model

import dms.domain.document.model.DocumentModel

/** One line of the document list. */
data class DocumentRowModel(
    val document: DocumentModel,
    val categoryName: String?,
)
