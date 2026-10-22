package dms.application.common

import dms.application.document.MoveDocument
import dms.application.library.CatalogStore
import dms.application.port.storage.model.StorageResult
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.RelativePathModel

/**
 * Files a document into a folder of the library, in every library folder: missing folders are
 * created on the way, and a name another entry of the target folder already has gets a ` (2)`
 * suffix, so filing never fails on a clash. Renaming within the same folder keeps the strict rule of
 * [MoveDocument] instead: a taken name is refused.
 */
class DocumentFiling(
    private val store: CatalogStore,
    private val names: SiblingNames,
    private val moveDocument: MoveDocument,
) {

    suspend fun file(documentId: DocumentIdModel, folder: RelativePathModel, fileName: String?): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val document = library.catalog.documents[documentId] ?: return OperationOutcome.NotFound
        val name = fileName ?: document.path.fileName
        val violation = (folder.segments + name).firstNotNullOfOrNull { FileNamePolicy.check(name = it) }
        if (violation != null) {
            return OperationOutcome.InvalidName(violation = violation)
        }
        val currentFolder = document.path.parent ?: RelativePathModel.ROOT
        if (folder == currentFolder) {
            return moveDocument(documentId = documentId, newPath = folder.resolve(name = name))
        }
        val siblings = names.of(library = library, parent = folder)
        if (siblings is StorageResult.Failure) {
            return OperationOutcome.StorageFailed(failure = siblings)
        }
        val free = FileNamePolicy.nextFreeName(name = name, siblings = (siblings as StorageResult.Success).value)
        return moveDocument(documentId = documentId, newPath = folder.resolve(name = free))
    }
}
