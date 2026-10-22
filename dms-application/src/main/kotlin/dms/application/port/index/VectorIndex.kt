package dms.application.port.index

import dms.application.port.index.model.DocumentVectorsModel
import dms.application.port.storage.model.StorageResult
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.LibraryFolderModel

/**
 * The embedding index of one library folder, one file per document in `.dms-index/`. Named by
 * document id, so moves and renames never touch it; replicated like the text store.
 */
interface VectorIndex {

    suspend fun write(folder: LibraryFolderModel, vectors: DocumentVectorsModel): StorageResult<Unit>

    /** The stored vectors, or `null` if there are none or the file cannot be read. */
    suspend fun read(folder: LibraryFolderModel, documentId: DocumentIdModel): DocumentVectorsModel?
}
