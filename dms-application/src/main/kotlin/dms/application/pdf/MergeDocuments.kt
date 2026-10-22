package dms.application.pdf

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.common.SiblingNames
import dms.application.common.model.LibrarySnapshotModel
import dms.application.document.TrashDocument
import dms.application.library.CatalogStore
import dms.application.port.pdf.PdfEditor
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageResult
import dms.application.port.system.Clock
import dms.application.port.system.IdGenerator
import dms.application.sync.Replicator
import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.RelativePathModel

/**
 * Merges PDF documents into a new one named [fileName], in the given order, in the folder of the
 * first. The new document takes over category, tags and correspondent of the first. The sources go
 * to the trash only with [trashSources], which the user confirmed.
 */
class MergeDocuments(
    private val store: CatalogStore,
    private val pdf: PdfEditor,
    private val replicator: Replicator,
    private val names: SiblingNames,
    private val trashDocument: TrashDocument,
    private val activity: ActivityRecorder,
    private val clock: Clock,
    private val ids: IdGenerator,
) {

    suspend operator fun invoke(
        documentIds: List<DocumentIdModel>,
        fileName: String,
        trashSources: Boolean,
    ): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val sources = documentIds.map { library.catalog.documents[it] ?: return OperationOutcome.NotFound }
        if (sources.size < 2 || sources.any { it.kind != DocumentKind.PDF }) {
            return OperationOutcome.NotFound
        }
        val first = sources.first()
        val path = when (val target = targetOf(library = library, first = first, fileName = fileName)) {
            is TargetResult.Refused -> return target.outcome
            is TargetResult.Free -> target.path
        }
        val written = pdf.merge(folder = library.folders.primary, sources = sources.map { it.path }, target = path)
        if (written is StorageResult.Failure) {
            return OperationOutcome.StorageFailed(failure = written)
        }
        val facts = (written as StorageResult.Success).value
        val pending = replicator.distribute(
            folders = library.folders,
            targets = library.folders.syncFolders,
            operation = FolderOperation.CopyFromPrimary(path = path),
        )
        val merged = DocumentModel(
            id = DocumentIdModel(value = ids.newId()),
            path = path,
            contentHash = checkNotNull(value = facts.contentHash),
            size = facts.size,
            addedAt = clock.now(),
            metadata = first.metadata.copy(title = null),
            categoryId = first.categoryId,
            tags = first.tags,
        )
        val outcome = store.commit { it.addDocument(document = merged) }.withPendingFolders(folders = pending)
        if (outcome !is OperationOutcome.Done) {
            return outcome
        }
        activity.record(kind = ActivityKind.PDF_EDITED, subject = "Merged ${sources.size} documents into $path")
        return if (trashSources) trash(sources = sources, merged = outcome) else outcome
    }

    /** A free path for the merged file next to [first], or why there is none. */
    private suspend fun targetOf(library: LibrarySnapshotModel, first: DocumentModel, fileName: String): TargetResult {
        val name = if (fileName.endsWith(suffix = ".pdf", ignoreCase = true)) fileName else "$fileName.pdf"
        val violation = FileNamePolicy.check(name = name)
        if (violation != null) {
            return TargetResult.Refused(outcome = OperationOutcome.InvalidName(violation = violation))
        }
        val folder = first.path.parent ?: RelativePathModel.ROOT
        return when (val siblings = names.of(library = library, parent = folder)) {
            is StorageResult.Failure -> TargetResult.Refused(
                outcome = OperationOutcome.StorageFailed(failure = siblings),
            )
            is StorageResult.Success -> TargetResult.Free(
                path = folder.resolve(name = FileNamePolicy.nextFreeName(name = name, siblings = siblings.value)),
            )
        }
    }

    private sealed interface TargetResult {
        data class Free(val path: RelativePathModel) : TargetResult
        data class Refused(val outcome: OperationOutcome) : TargetResult
    }

    /** Moves the sources to the trash one by one; the first refusal stops and is returned. */
    private suspend fun trash(sources: List<DocumentModel>, merged: OperationOutcome.Done): OperationOutcome {
        var pending = merged.pendingFolders
        var last: OperationOutcome = merged
        for (source in sources) {
            last = trashDocument(documentId = source.id, confirmed = true)
            if (last !is OperationOutcome.Done) {
                return last
            }
            pending = pending + last.pendingFolders
        }
        return last.withPendingFolders(folders = pending)
    }
}
