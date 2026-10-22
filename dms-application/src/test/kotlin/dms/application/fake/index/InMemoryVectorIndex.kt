package dms.application.fake.index

import dms.application.port.index.VectorIndex
import dms.application.port.index.model.DocumentVectorsModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFolderModel

/** `.dms-index/` of every library folder, in memory; folders in [unreachable] fail. */
class InMemoryVectorIndex(private val unreachable: Set<LibraryFolderIdModel>) : VectorIndex {

    private val entries = mutableMapOf<LibraryFolderIdModel, MutableMap<DocumentIdModel, DocumentVectorsModel>>()
    var writes: Int = 0
        private set

    override suspend fun write(folder: LibraryFolderModel, vectors: DocumentVectorsModel): StorageResult<Unit> {
        if (folder.id in unreachable) {
            return StorageResult.Failure(reason = StorageFailure.FOLDER_UNREACHABLE)
        }
        writes++
        entries.getOrPut(key = folder.id) { mutableMapOf() }[vectors.documentId] = vectors
        return StorageResult.Success(value = Unit)
    }

    override suspend fun read(folder: LibraryFolderModel, documentId: DocumentIdModel): DocumentVectorsModel? {
        return entries[folder.id]?.get(key = documentId)
    }
}
