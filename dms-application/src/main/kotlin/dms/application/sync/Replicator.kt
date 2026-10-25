package dms.application.sync

import dms.application.port.index.VectorIndex
import dms.application.port.plan.PlanStore
import dms.application.port.storage.CatalogRepository
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.OperationJournal
import dms.application.port.storage.TextStore
import dms.application.port.storage.model.CatalogLoad
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.PendingOperationModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.application.port.system.Clock
import dms.application.port.system.IdGenerator
import dms.application.sync.model.Replication
import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.CatalogRevisionModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.LibraryFoldersModel
import dms.domain.library.model.RelativePathModel

/**
 * Applies changes to every library folder: the primary first, then each sync folder. A failed primary
 * aborts before any sync folder is touched; a failed sync folder is recorded in the journal and the
 * others continue.
 */
class Replicator(
    private val storage: DocumentStorage,
    private val catalogs: CatalogRepository,
    private val texts: TextStore,
    private val plans: PlanStore,
    private val index: VectorIndex,
    private val journal: OperationJournal,
    private val clock: Clock,
    private val ids: IdGenerator,
) {

    /** Runs [onPrimary] on the primary, then brings every sync folder in line with [followUp]. */
    suspend fun <T> replicate(
        folders: LibraryFoldersModel,
        onPrimary: suspend (LibraryFolderModel) -> StorageResult<T>,
        followUp: FolderOperation,
    ): Replication<T> {
        return when (val result = onPrimary(folders.primary)) {
            is StorageResult.Failure -> Replication.PrimaryFailed(failure = result)
            is StorageResult.Success -> Replication.Done(
                value = result.value,
                pendingFolders = distribute(folders = folders, targets = folders.syncFolders, operation = followUp),
            )
        }
    }

    /**
     * Saves [catalog] to every folder. [basedOn] is the newest revision the change was made on; a primary
     * that holds a newer one fails with [StorageFailure.CATALOG_CHANGED] and nothing is written.
     */
    suspend fun saveCatalog(
        folders: LibraryFoldersModel,
        catalog: CatalogModel,
        basedOn: CatalogRevisionModel,
    ): Replication<Unit> {
        return replicate(
            folders = folders,
            onPrimary = { primary -> catalogs.save(folder = primary, catalog = catalog, basedOn = basedOn) },
            followUp = FolderOperation.SaveCatalog,
        )
    }

    /** Moves [path] to the trash in every folder. A folder where it is already gone counts as done. */
    suspend fun trash(folders: LibraryFoldersModel, path: RelativePathModel): Replication<Unit> {
        return replicate(
            folders = folders,
            onPrimary = { primary -> storage.moveToTrash(folder = primary, path = path).orIfMissing() },
            followUp = FolderOperation.MoveToTrash(path = path),
        )
    }

    /**
     * Applies [operation] to each of [targets] and returns the ids of the folders where it failed.
     * Failures are recorded in the journal.
     */
    suspend fun distribute(
        folders: LibraryFoldersModel,
        targets: List<LibraryFolderModel>,
        operation: FolderOperation,
    ): Set<LibraryFolderIdModel> {
        return targets.filter { target ->
            val result = execute(primary = folders.primary, target = target, operation = operation)
            if (result is StorageResult.Failure) {
                record(folderId = target.id, operation = operation)
            }
            result is StorageResult.Failure
        }.map { it.id }.toSet()
    }

    /** Executes one operation against one sync folder, reading whatever it needs from the primary. */
    suspend fun execute(
        primary: LibraryFolderModel,
        target: LibraryFolderModel,
        operation: FolderOperation,
    ): StorageResult<Unit> {
        return when (operation) {
            is FolderOperation.CopyFromPrimary -> copy(primary = primary, target = target, operation = operation)

            is FolderOperation.CreateFolder -> storage.createFolder(folder = target, path = operation.path)
            is FolderOperation.Move -> storage.move(folder = target, from = operation.from, to = operation.to)
            is FolderOperation.MoveToTrash -> storage.moveToTrash(folder = target, path = operation.path).orIfMissing()
            is FolderOperation.CopyText -> copyText(primary = primary, target = target, operation = operation)
            is FolderOperation.CopyPlan -> copyPlan(primary = primary, target = target, name = operation.name)
            is FolderOperation.CopyIndex -> copyIndex(primary = primary, target = target, operation = operation)
            FolderOperation.SaveCatalog -> copyCatalog(primary = primary, target = target)
        }
    }

    /** A copy that finds the same content already in place has reached its goal. */
    private suspend fun copy(
        primary: LibraryFolderModel,
        target: LibraryFolderModel,
        operation: FolderOperation.CopyFromPrimary,
    ): StorageResult<Unit> {
        val copied = storage.copyFile(
            from = primary,
            to = target,
            path = operation.path,
            replaceExisting = operation.replaceExisting,
        ).discardValue()
        if (copied is StorageResult.Failure && copied.reason == StorageFailure.ALREADY_EXISTS) {
            val source = storage.readFacts(folder = primary, path = operation.path)
            val existing = storage.readFacts(folder = target, path = operation.path)
            if (source is StorageResult.Success && source == existing) {
                return StorageResult.Success(value = Unit)
            }
        }
        return copied
    }

    private suspend fun copyText(
        primary: LibraryFolderModel,
        target: LibraryFolderModel,
        operation: FolderOperation.CopyText,
    ): StorageResult<Unit> {
        val text = texts.read(folder = primary, documentId = operation.documentId)
            ?: return StorageResult.Failure(reason = StorageFailure.NOT_FOUND)
        return texts.write(folder = target, documentId = operation.documentId, text = text)
    }

    private suspend fun copyPlan(
        primary: LibraryFolderModel,
        target: LibraryFolderModel,
        name: String,
    ): StorageResult<Unit> {
        val record = plans.read(folder = primary, name = name)
            ?: return StorageResult.Failure(reason = StorageFailure.NOT_FOUND)
        return plans.write(folder = target, record = record)
    }

    private suspend fun copyIndex(
        primary: LibraryFolderModel,
        target: LibraryFolderModel,
        operation: FolderOperation.CopyIndex,
    ): StorageResult<Unit> {
        val vectors = index.read(folder = primary, documentId = operation.documentId)
            ?: return StorageResult.Failure(reason = StorageFailure.NOT_FOUND)
        return index.write(folder = target, vectors = vectors)
    }

    /**
     * Never lowers a sync folder's revision: a sync folder holding a newer catalog fails with
     * [StorageFailure.CATALOG_CHANGED] and stays in the journal until opening the library merged it.
     */
    private suspend fun copyCatalog(primary: LibraryFolderModel, target: LibraryFolderModel): StorageResult<Unit> {
        return when (val load = catalogs.load(folder = primary)) {
            is CatalogLoad.Found -> catalogs.save(
                folder = target,
                catalog = load.catalog,
                basedOn = load.catalog.revision,
            )

            CatalogLoad.NotFound -> StorageResult.Failure(reason = StorageFailure.NOT_FOUND)
            is CatalogLoad.Failed -> load.failure
        }
    }

    private suspend fun record(folderId: LibraryFolderIdModel, operation: FolderOperation) {
        journal.record(
            operation = PendingOperationModel(
                id = ids.newId(),
                folderId = folderId,
                operation = operation,
                recordedAt = clock.now(),
            ),
        )
    }

    /** Trashing something that is already gone has reached its goal. */
    private fun StorageResult<Unit>.orIfMissing(): StorageResult<Unit> {
        if (this is StorageResult.Failure && reason == StorageFailure.NOT_FOUND) {
            return StorageResult.Success(value = Unit)
        }
        return this
    }

    private fun <T> StorageResult<T>.discardValue(): StorageResult<Unit> {
        return when (this) {
            is StorageResult.Success -> StorageResult.Success(value = Unit)
            is StorageResult.Failure -> this
        }
    }
}
