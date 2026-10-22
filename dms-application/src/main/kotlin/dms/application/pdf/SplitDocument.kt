package dms.application.pdf

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.common.SiblingNames
import dms.application.common.model.LibrarySnapshotModel
import dms.application.document.TrashDocument
import dms.application.library.CatalogStore
import dms.application.port.pdf.PdfEditor
import dms.application.port.pdf.model.PageChangeModel
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageResult
import dms.application.port.system.Clock
import dms.application.port.system.IdGenerator
import dms.application.sync.Replicator
import dms.domain.catalog.CatalogUpdate
import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel

/**
 * Splits a PDF into new documents, one per page range (pages counted from 0), next to the original.
 * Each new document takes over category, tags and correspondent. The original goes to the trash only
 * with [trashOriginal], which the user confirmed.
 */
class SplitDocument(
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
        documentId: DocumentIdModel,
        ranges: List<IntRange>,
        trashOriginal: Boolean,
    ): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val original = library.catalog.documents[documentId] ?: return OperationOutcome.NotFound
        val validRanges = ranges.isNotEmpty() && ranges.none { it.isEmpty() || it.first < 0 }
        if (original.kind != DocumentKind.PDF || !validRanges) {
            return OperationOutcome.NotFound
        }
        val created = mutableListOf<DocumentModel>()
        val pending = mutableSetOf<LibraryFolderIdModel>()
        ranges.forEach { range ->
            when (val part = writePart(
                library = library,
                original = original,
                range = range,
                taken = created.map { it.path.fileName },
            )) {
                is PartResult.Failed -> return part.outcome
                is PartResult.Written -> {
                    created += part.document
                    pending += part.pending
                }
            }
        }
        val outcome = store.commit { catalog ->
            val start: CatalogUpdate = CatalogUpdate.Applied(catalog = catalog)
            created.fold(initial = start) { update, document ->
                update.andThen { it.addDocument(document = document) }
            }
        }.withPendingFolders(folders = pending)
        if (outcome !is OperationOutcome.Done) {
            return outcome
        }
        activity.record(
            kind = ActivityKind.PDF_EDITED,
            subject = "Split ${original.path} into ${created.size} documents",
        )
        return if (trashOriginal) trashDocument(documentId = original.id, confirmed = true) else outcome
    }

    private suspend fun writePart(
        library: LibrarySnapshotModel,
        original: DocumentModel,
        range: IntRange,
        taken: List<String>,
    ): PartResult {
        val folder = original.path.parent ?: RelativePathModel.ROOT
        val siblings = names.of(library = library, parent = folder)
        if (siblings is StorageResult.Failure) {
            return PartResult.Failed(outcome = OperationOutcome.StorageFailed(failure = siblings))
        }
        val base = original.path.fileName.substringBeforeLast(delimiter = '.')
        val label = if (range.first == range.last) {
            "page ${range.first + 1}"
        } else {
            "pages ${range.first + 1}-${range.last + 1}"
        }
        val name = FileNamePolicy.nextFreeName(
            name = FileNamePolicy.sanitize(name = "$base ($label).pdf"),
            siblings = (siblings as StorageResult.Success).value + taken,
        )
        val path = folder.resolve(name = name)
        val written = pdf.rewrite(
            folder = library.folders.primary,
            source = original.path,
            pages = range.map { PageChangeModel(page = it) },
            target = path,
        )
        if (written is StorageResult.Failure) {
            return PartResult.Failed(outcome = OperationOutcome.StorageFailed(failure = written))
        }
        val facts = (written as StorageResult.Success).value
        val pending = replicator.distribute(
            folders = library.folders,
            targets = library.folders.syncFolders,
            operation = FolderOperation.CopyFromPrimary(path = path),
        )
        val document = DocumentModel(
            id = DocumentIdModel(value = ids.newId()),
            path = path,
            contentHash = checkNotNull(value = facts.contentHash),
            size = facts.size,
            addedAt = clock.now(),
            metadata = original.metadata.copy(title = original.metadata.title?.let { "$it ($label)" }),
            categoryId = original.categoryId,
            tags = original.tags,
        )
        return PartResult.Written(document = document, pending = pending)
    }

    private sealed interface PartResult {
        data class Written(val document: DocumentModel, val pending: Set<LibraryFolderIdModel>) : PartResult
        data class Failed(val outcome: OperationOutcome) : PartResult
    }
}
