package dms.application.search

import dms.application.common.model.LibrarySnapshotModel
import dms.application.port.ai.DocumentContentReader
import dms.application.port.storage.TextStore
import dms.application.port.system.Clock
import dms.application.search.model.IndexedTextModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.QueryContextModel
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.util.Locale

/**
 * Matches documents against their text, ignoring case. A document's text is what the AI transcribed of
 * it, kept in the text store, plus the text layer of its file, plus the user's note. File texts are read
 * once and kept in memory until the file changes, so only the first search reads every file. Shared by
 * every use case that searches, so they share that memory. Never writes.
 */
class ContentMatcher(
    private val reader: DocumentContentReader,
    private val texts: TextStore,
    private val clock: Clock,
) {

    private val mutex = Mutex()
    private val index = HashMap<DocumentIdModel, IndexedTextModel>()

    /** The documents whose text contains [wanted]; nothing for a blank [wanted]. */
    suspend fun containing(library: LibrarySnapshotModel, wanted: String): Set<DocumentIdModel> {
        val normalized = wanted.trim().lowercase(Locale.ROOT)
        if (normalized.isEmpty()) {
            return emptySet()
        }
        return mutex.withLock {
            library.catalog.documents.values
                .filter { containsText(folder = library.folders.primary, document = it, wanted = normalized) }
                .map { it.id }
                .toSet()
        }
    }

    /**
     * The documents [query] matches, by path. Its content term is looked up in the documents' texts,
     * only for the documents the other conditions leave.
     */
    suspend fun matching(library: LibrarySnapshotModel, query: DocumentQueryModel): List<DocumentModel> {
        val context = QueryContextModel(catalog = library.catalog, today = clock.today(), zone = clock.zone())
        val candidates = query.filter(context = context)
        val wanted = query.contentText?.trim()?.lowercase(Locale.ROOT)
        if (wanted.isNullOrEmpty()) {
            return candidates
        }
        return mutex.withLock {
            candidates.filter { containsText(folder = library.folders.primary, document = it, wanted = wanted) }
        }
    }

    private suspend fun containsText(folder: LibraryFolderModel, document: DocumentModel, wanted: String): Boolean {
        val note = document.note?.lowercase(Locale.ROOT).orEmpty()
        return note.contains(other = wanted) || fileText(folder = folder, document = document).contains(other = wanted)
    }

    private suspend fun fileText(folder: LibraryFolderModel, document: DocumentModel): String {
        val known = index[document.id]?.takeIf { it.contentHash == document.contentHash }
        if (known != null) {
            return known.text
        }
        val stored = texts.read(folder = folder, documentId = document.id)
        val extracted = reader.text(folder = folder, document = document)
        val text = listOfNotNull(stored, extracted).joinToString(separator = "\n").lowercase(Locale.ROOT)
        index[document.id] = IndexedTextModel(contentHash = document.contentHash, text = text)
        return text
    }
}
