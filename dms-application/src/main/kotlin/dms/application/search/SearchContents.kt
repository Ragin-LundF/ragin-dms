package dms.application.search

import dms.application.library.CatalogStore
import dms.application.port.ai.DocumentContentReader
import dms.application.port.storage.TextStore
import dms.application.search.model.IndexedText
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.library.LibraryFolder
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.util.Locale

/**
 * Finds the documents whose text contains a query, ignoring case. A document's text is what the AI
 * transcribed of it, kept in the text store, plus the text layer of its file. Both are read once and
 * kept in memory until the file changes, so only the first search reads every file. Never writes.
 */
class SearchContents(
    private val store: CatalogStore,
    private val reader: DocumentContentReader,
    private val texts: TextStore,
) {

    private val mutex = Mutex()
    private val index = HashMap<DocumentId, IndexedText>()

    suspend operator fun invoke(query: String): Set<DocumentId> {
        val wanted = query.trim().lowercase(Locale.ROOT)
        val library = store.current()
        if (wanted.isEmpty() || library == null) {
            return emptySet()
        }
        return mutex.withLock {
            library.catalog.documents.values
                .filter { document -> textOf(folder = library.folders.primary, document = document).contains(wanted) }
                .map { it.id }
                .toSet()
        }
    }

    private suspend fun textOf(folder: LibraryFolder, document: Document): String {
        val known = index[document.id]?.takeIf { it.contentHash == document.contentHash }
        if (known != null) {
            return known.text
        }
        val stored = texts.read(folder = folder, documentId = document.id)
        val extracted = reader.text(folder = folder, document = document)
        val text = listOfNotNull(stored, extracted).joinToString(separator = "\n").lowercase(Locale.ROOT)
        index[document.id] = IndexedText(contentHash = document.contentHash, text = text)
        return text
    }
}
