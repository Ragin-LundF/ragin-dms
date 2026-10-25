package dms.application.pdf

import dms.application.common.OperationOutcome
import dms.application.common.model.LibrarySnapshotModel
import dms.application.library.CatalogStore
import dms.application.port.ai.DocumentContentReader
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.TextStore
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageResult
import dms.application.port.system.IdGenerator
import dms.application.sync.Replicator
import dms.domain.change.model.FileFactsModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.TemporaryNames
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel

/**
 * A changed PDF is always a new version of its document: written next to it under a temporary name,
 * put in place in the primary with the old version going to the system trash, copied over the old
 * version in every sync folder (which trashes it there too), and recorded in the catalog. The text the
 * search uses is read anew from the new file.
 */
class PdfVersions(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val replicator: Replicator,
    private val reader: DocumentContentReader,
    private val texts: TextStore,
    private val ids: IdGenerator,
) {

    /** Replaces [document]'s file with what [write] creates at the temporary path it is given. */
    suspend fun replace(
        library: LibrarySnapshotModel,
        document: DocumentModel,
        write: suspend (LibraryFolderModel, RelativePathModel) -> StorageResult<FileFactsModel>,
    ): OperationOutcome {
        val primary = library.folders.primary
        val temporary = TemporaryNames.siblingOf(path = document.path, uniqueId = ids.newId())
        val written = write(primary, temporary)
        if (written is StorageResult.Failure) {
            return OperationOutcome.StorageFailed(failure = written)
        }
        val replaced = storage.replaceWith(folder = primary, target = document.path, replacement = temporary)
        if (replaced is StorageResult.Failure) {
            storage.moveToTrash(folder = primary, path = temporary)
            return OperationOutcome.StorageFailed(failure = replaced)
        }
        val facts = (replaced as StorageResult.Success).value
        val pending = replicator.distribute(
            folders = library.folders,
            targets = library.folders.syncFolders,
            operation = FolderOperation.CopyFromPrimary(path = document.path, replaceExisting = true),
        )
        val contentHash = checkNotNull(value = facts.contentHash) { "A written file is always hashed" }
        val outcome = store
            .commit { it.updateContent(documentId = document.id, contentHash = contentHash, size = facts.size) }
            .withPendingFolders(folders = pending)
        if (outcome is OperationOutcome.Done) {
            refreshText(library = library, document = outcome.catalog.documents[document.id] ?: document)
        }
        return outcome
    }

    /** Derived data: when the new file has a text layer, it is what search finds from now on. */
    private suspend fun refreshText(library: LibrarySnapshotModel, document: DocumentModel) {
        val text = reader.text(folder = library.folders.primary, document = document)
            ?.takeIf { it.isNotBlank() }
            ?: return
        replicator.replicate(
            folders = library.folders,
            onPrimary = { primary -> texts.write(folder = primary, documentId = document.id, text = text) },
            followUp = FolderOperation.CopyText(documentId = document.id),
        )
    }
}
