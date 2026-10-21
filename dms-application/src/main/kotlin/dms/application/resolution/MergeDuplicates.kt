package dms.application.resolution

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.TaskStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.Catalog
import dms.domain.catalog.CatalogUpdate
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.document.DocumentMetadata
import dms.domain.library.LibraryFolder

/**
 * Keeps one of several catalogued copies of a file. The others go to the system trash in every
 * folder and leave the catalog; what the user assigned to them (category, tags, title, date,
 * correspondent, the AI classification) moves to the kept document where it has nothing yet.
 *
 * Only true copies go: the catalog must know them with the same size and SHA-256, and right before
 * anything moves, each folder compares the two files byte by byte. If a copy differs in the primary,
 * nothing changes at all. A sync folder where the comparison fails, or that is offline, keeps its
 * file, and the next scan reports it again. Only with the user's confirmation.
 */
class MergeDuplicates(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val tasks: TaskStore,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(keep: DocumentId, confirmed: Boolean): OperationOutcome {
        if (!confirmed) {
            return OperationOutcome.NotConfirmed
        }
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val kept = library.catalog.documents[keep] ?: return OperationOutcome.NotFound
        val copies = library.catalog.findByContentHash(contentHash = kept.contentHash)
            .filter { it.id != kept.id && it.size == kept.size }
        val primary = library.folders.primary
        if (copies.isEmpty() || !copies.all { identical(folder = primary, kept = kept, copy = it) }) {
            return OperationOutcome.ScanOutdated
        }
        copies.forEach { copy ->
            val trashed = storage.moveToTrash(folder = primary, path = copy.path)
            if (trashed is StorageResult.Failure) {
                return OperationOutcome.StorageFailed(failure = trashed)
            }
            library.folders.syncFolders.filter { identical(folder = it, kept = kept, copy = copy) }.forEach {
                storage.moveToTrash(folder = it, path = copy.path)
            }
        }
        val outcome = store.commit { catalog -> merged(catalog = catalog, kept = kept, copies = copies) }
        if (outcome is OperationOutcome.Done) {
            copies.forEach { copy ->
                tasks.removeSuggestion(documentId = copy.id)
                activity.record(kind = ActivityKind.DOCUMENT_TRASHED, subject = copy.path.value)
            }
        }
        return outcome
    }

    private suspend fun identical(folder: LibraryFolder, kept: Document, copy: Document): Boolean {
        val same = storage.sameContent(folder = folder, first = kept.path, second = copy.path)
        return (same as? StorageResult.Success)?.value == true
    }

    private fun merged(catalog: Catalog, kept: Document, copies: List<Document>): CatalogUpdate {
        val metadata = DocumentMetadata(
            title = kept.metadata.title ?: copies.firstNotNullOfOrNull { it.metadata.title },
            documentDate = kept.metadata.documentDate ?: copies.firstNotNullOfOrNull { it.metadata.documentDate },
            correspondent = kept.metadata.correspondent ?: copies.firstNotNullOfOrNull { it.metadata.correspondent },
        )
        val categoryId = kept.categoryId ?: copies.firstNotNullOfOrNull { it.categoryId }
        val classification = kept.classification ?: copies.firstNotNullOfOrNull { it.classification }
        val start: CatalogUpdate = CatalogUpdate.Applied(catalog = catalog)
        val removed = copies.fold(initial = start) { update, copy ->
            update.andThen { it.removeDocument(documentId = copy.id) }
        }
        return removed
            .andThen { it.updateMetadata(documentId = kept.id, metadata = metadata) }
            .andThen { it.assignCategory(documentId = kept.id, categoryId = categoryId) }
            .andThen { it.setTags(documentId = kept.id, newTags = kept.tags + copies.flatMap { copy -> copy.tags }) }
            .andThen { current ->
                classification?.let { current.recordClassification(documentId = kept.id, record = it) }
                    ?: CatalogUpdate.Applied(catalog = current)
            }
    }
}
