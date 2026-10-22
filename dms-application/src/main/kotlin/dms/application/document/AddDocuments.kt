package dms.application.document

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.common.SiblingNames
import dms.application.common.model.LibrarySnapshotModel
import dms.application.document.model.AddedFileModel
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.ExternalFileModel
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.application.port.system.Clock
import dms.application.port.system.IdGenerator
import dms.application.sync.Replicator
import dms.application.sync.model.Replication
import dms.domain.document.FileNameDates
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.RelativePathModel

/**
 * Copies files from outside the library into [targetFolder] of every library folder and catalogs
 * them. The original name is kept; a taken name gets a ` (2)` suffix. Files with content the library
 * already has are added as well and reported as duplicates.
 */
class AddDocuments(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val replicator: Replicator,
    private val names: SiblingNames,
    private val activity: ActivityRecorder,
    private val clock: Clock,
    private val ids: IdGenerator,
) {

    suspend operator fun invoke(files: List<ExternalFileModel>, targetFolder: RelativePathModel): List<AddedFileModel> {
        return files.map { add(file = it, targetFolder = targetFolder) }
    }

    private suspend fun add(file: ExternalFileModel, targetFolder: RelativePathModel): AddedFileModel {
        val library = store.current() ?: return AddedFileModel(source = file, outcome = OperationOutcome.LibraryNotOpen)
        val violation = FileNamePolicy.check(name = file.fileName)
        if (violation != null) {
            return AddedFileModel(source = file, outcome = OperationOutcome.InvalidName(violation = violation))
        }
        val siblings = names.of(library = library, parent = targetFolder)
        if (siblings is StorageResult.Failure) {
            return AddedFileModel(source = file, outcome = OperationOutcome.StorageFailed(failure = siblings))
        }
        val freeName = FileNamePolicy.nextFreeName(
            name = file.fileName,
            siblings = (siblings as StorageResult.Success).value,
        )
        val target = targetFolder.resolve(name = freeName)
        val outcome = copyAndCatalog(library = library, file = file, target = target)
        if (outcome !is OperationOutcome.Done) {
            return AddedFileModel(source = file, outcome = outcome)
        }
        activity.record(kind = ActivityKind.DOCUMENT_ADDED, subject = target.value)
        val added = outcome.catalog.findByPath(path = target)
        val duplicates = added?.let { document ->
            outcome.catalog.findByContentHash(contentHash = document.contentHash).filter { it.id != document.id }
        }
        return AddedFileModel(source = file, outcome = outcome, path = target, duplicates = duplicates.orEmpty())
    }

    /** Writes the file to every folder first and the catalog second. */
    private suspend fun copyAndCatalog(
        library: LibrarySnapshotModel,
        file: ExternalFileModel,
        target: RelativePathModel,
    ): OperationOutcome {
        val replication = replicator.replicate(
            folders = library.folders,
            onPrimary = { primary -> storage.importFile(folder = primary, source = file, target = target) },
            followUp = FolderOperation.CopyFromPrimary(path = target),
        )
        if (replication is Replication.PrimaryFailed) {
            return OperationOutcome.StorageFailed(failure = replication.failure)
        }
        val copied = replication as Replication.Done
        val contentHash = copied.value.contentHash
            ?: return OperationOutcome.StorageFailed(failure = StorageResult.Failure(reason = StorageFailure.IO_ERROR))
        val document = DocumentModel(
            id = DocumentIdModel(value = ids.newId()),
            path = target,
            contentHash = contentHash,
            size = copied.value.size,
            addedAt = clock.now(),
            // The copy was just created, so only the name can tell how old the document is.
            metadata = DocumentMetadataModel(documentDate = FileNameDates.dateIn(name = target.fileName)),
        )
        return store.commit { it.addDocument(document = document) }
            .withPendingFolders(folders = copied.pendingFolders)
    }
}
