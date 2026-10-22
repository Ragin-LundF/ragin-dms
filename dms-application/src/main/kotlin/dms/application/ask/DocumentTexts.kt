package dms.application.ask

import dms.application.port.ai.DocumentContentReader
import dms.application.port.storage.TextStore
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.LibraryFolderModel
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * The text of documents as written, for the index and the answers: the AI's transcription from the
 * text store, or the file's text layer. Read once per file version and kept in memory.
 */
class DocumentTexts(
    private val reader: DocumentContentReader,
    private val texts: TextStore,
) {

    private val mutex = Mutex()
    private val cache = HashMap<DocumentIdModel, Pair<ContentHashModel, String>>()

    suspend fun of(folder: LibraryFolderModel, document: DocumentModel): String {
        return mutex.withLock {
            cache[document.id]?.takeIf { it.first == document.contentHash }?.second ?: read(
                folder = folder,
                document = document,
            )
        }
    }

    private suspend fun read(folder: LibraryFolderModel, document: DocumentModel): String {
        val stored = texts.read(folder = folder, documentId = document.id)
        val text = stored?.takeIf { it.isNotBlank() }
            ?: reader.text(folder = folder, document = document).orEmpty()
        val cleaned = text.replace(oldChar = '\u000C', newChar = '\n')
        cache[document.id] = document.contentHash to cleaned
        return cleaned
    }
}
