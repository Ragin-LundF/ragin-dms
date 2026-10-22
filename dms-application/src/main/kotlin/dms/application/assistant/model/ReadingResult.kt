package dms.application.assistant.model

import dms.application.port.ai.model.DocumentContent
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel

/** A document read for the AI, or why it could not be read. */
sealed interface ReadingResult {

    data class Read(
        val document: DocumentModel,
        val catalog: CatalogModel,
        val content: DocumentContent,
    ) : ReadingResult

    data object LibraryNotOpen : ReadingResult

    data object NotFound : ReadingResult

    /** The file type is never sent to the AI, or the file could not be read. [detail] is technical. */
    data class NotReadable(val detail: String?) : ReadingResult
}
