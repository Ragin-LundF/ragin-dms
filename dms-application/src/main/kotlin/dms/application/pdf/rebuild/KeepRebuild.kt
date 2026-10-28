package dms.application.pdf.rebuild

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.common.SiblingNames
import dms.application.common.model.LibrarySnapshotModel
import dms.application.library.CatalogStore
import dms.application.pdf.PdfVersions
import dms.application.pdf.rebuild.model.RebuildChoice
import dms.application.port.pdf.PdfRebuilder
import dms.application.port.pdf.model.RebuiltFileModel
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageResult
import dms.application.port.system.Clock
import dms.application.port.system.IdGenerator
import dms.application.sync.Replicator
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.RelativePathModel

/**
 * Applies the user's choice for a rebuilt PDF. Keeping the original changes nothing. Replacing makes the
 * rebuild a new version of the document, with the scan going to the system trash. Keeping both adds the
 * rebuild as a new document next to the original, with its category, tags and fields, file first and
 * catalog second. A scan that changed after the conversion is never replaced: its newer content would
 * go to the trash unseen ([OperationOutcome.SourceChanged]); keeping both still works. The rebuild is
 * deleted once it is placed, not wanted, or has no document or library left to go to; after any other
 * failure it stays, so the user can choose again.
 */
class KeepRebuild(
    private val store: CatalogStore,
    private val rebuilder: PdfRebuilder,
    private val versions: PdfVersions,
    private val replicator: Replicator,
    private val names: SiblingNames,
    private val activity: ActivityRecorder,
    private val clock: Clock,
    private val ids: IdGenerator,
) {

    suspend operator fun invoke(
        documentId: DocumentIdModel,
        rebuilt: RebuiltFileModel,
        choice: RebuildChoice,
    ): OperationOutcome {
        val outcome = apply(documentId = documentId, rebuilt = rebuilt, choice = choice)
        if (outcome is OperationOutcome.Done || outcome in GONE) {
            rebuilder.discard(rebuilt = rebuilt)
        }
        return outcome
    }

    private suspend fun apply(
        documentId: DocumentIdModel,
        rebuilt: RebuiltFileModel,
        choice: RebuildChoice,
    ): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val document = library.catalog.documents[documentId] ?: return OperationOutcome.NotFound
        return when (choice) {
            RebuildChoice.KEEP_ORIGINAL -> OperationOutcome.Done(catalog = library.catalog, pendingFolders = emptySet())
            RebuildChoice.REPLACE_WITH_REBUILD -> if (document.contentHash == rebuilt.sourceHash) {
                replace(library = library, document = document, rebuilt = rebuilt)
            } else {
                OperationOutcome.SourceChanged
            }
            RebuildChoice.KEEP_BOTH -> addCopy(library = library, document = document, rebuilt = rebuilt)
        }
    }

    private suspend fun replace(
        library: LibrarySnapshotModel,
        document: DocumentModel,
        rebuilt: RebuiltFileModel,
    ): OperationOutcome {
        val outcome = versions.replace(library = library, document = document) { folder, target ->
            rebuilder.place(rebuilt = rebuilt, folder = folder, target = target)
        }
        if (outcome is OperationOutcome.Done) {
            activity.record(kind = ActivityKind.PDF_EDITED, subject = "Rebuilt: ${document.path}")
        }
        return outcome
    }

    private suspend fun addCopy(
        library: LibrarySnapshotModel,
        document: DocumentModel,
        rebuilt: RebuiltFileModel,
    ): OperationOutcome {
        val folder = document.path.parent ?: RelativePathModel.ROOT
        val siblings = when (val listed = names.of(library = library, parent = folder)) {
            is StorageResult.Failure -> return OperationOutcome.StorageFailed(failure = listed)
            is StorageResult.Success -> listed.value
        }
        val base = document.path.fileName.substringBeforeLast(delimiter = '.')
        val name = FileNamePolicy.nextFreeName(
            name = FileNamePolicy.sanitize(name = "$base (rebuilt).pdf"),
            siblings = siblings,
        )
        val path = folder.resolve(name = name)
        val placed = rebuilder.place(rebuilt = rebuilt, folder = library.folders.primary, target = path)
        val facts = when (placed) {
            is StorageResult.Failure -> return OperationOutcome.StorageFailed(failure = placed)
            is StorageResult.Success -> placed.value
        }
        val pending = replicator.distribute(
            folders = library.folders,
            targets = library.folders.syncFolders,
            operation = FolderOperation.CopyFromPrimary(path = path),
        )
        val copy = document.copy(
            id = DocumentIdModel(value = ids.newId()),
            path = path,
            contentHash = checkNotNull(value = facts.contentHash) { "A written file is always hashed" },
            size = facts.size,
            addedAt = clock.now(),
            classification = null,
        )
        val outcome = store.commit { it.addDocument(document = copy) }.withPendingFolders(folders = pending)
        if (outcome is OperationOutcome.Done) {
            activity.record(kind = ActivityKind.PDF_EDITED, subject = "Rebuilt as ${copy.path}")
        }
        return outcome
    }

    private companion object {
        /** Outcomes after which the rebuild has nowhere to go any more. */
        val GONE = setOf(OperationOutcome.NotFound, OperationOutcome.LibraryNotOpen)
    }
}
