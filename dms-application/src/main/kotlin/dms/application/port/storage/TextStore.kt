package dms.application.port.storage

import dms.application.port.storage.model.StorageResult
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.LibraryFolderModel

/** The extracted text of documents, stored in `.dms-text/` of a library folder for search. */
interface TextStore {

    suspend fun write(folder: LibraryFolderModel, documentId: DocumentIdModel, text: String): StorageResult<Unit>

    /** The stored text, or `null` if there is none. */
    suspend fun read(folder: LibraryFolderModel, documentId: DocumentIdModel): String?
}
