package dms.application.port.storage

import dms.application.port.storage.model.StorageResult
import dms.domain.document.DocumentId
import dms.domain.library.LibraryFolder

/** The extracted text of documents, stored in `.dms-text/` of a library folder for search. */
interface TextStore {

    suspend fun write(folder: LibraryFolder, documentId: DocumentId, text: String): StorageResult<Unit>

    /** The stored text, or `null` if there is none. */
    suspend fun read(folder: LibraryFolder, documentId: DocumentId): String?
}
