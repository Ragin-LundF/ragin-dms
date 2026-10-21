package dms.application.assistant

import dms.application.assistant.model.ReadingResult
import dms.application.library.CatalogStore
import dms.application.port.ai.DocumentContentReader
import dms.application.port.ai.model.DocumentContent
import dms.domain.document.DocumentId

/** Reads a document of the open library from the primary folder, for the AI. Never changes it. */
class DocumentReading(
    private val store: CatalogStore,
    private val reader: DocumentContentReader,
) {

    suspend fun read(documentId: DocumentId): ReadingResult {
        val library = store.current() ?: return ReadingResult.LibraryNotOpen
        val document = library.catalog.documents[documentId] ?: return ReadingResult.NotFound
        if (!document.kind.readableByAi) {
            return ReadingResult.NotReadable(detail = null)
        }
        return when (val content = reader.read(folder = library.folders.primary, document = document)) {
            is DocumentContent.NotReadable -> ReadingResult.NotReadable(detail = content.reason)
            else -> ReadingResult.Read(document = document, catalog = library.catalog, content = content)
        }
    }
}
