package dms.application.library

import dms.application.common.ActivityRecorder
import dms.application.common.DocumentDates
import dms.application.common.OperationOutcome
import dms.application.library.model.ScannedFile
import dms.application.port.state.model.ActivityKind
import dms.application.port.system.Clock
import dms.application.port.system.IdGenerator
import dms.domain.catalog.Catalog
import dms.domain.catalog.CatalogRejection
import dms.domain.catalog.CatalogUpdate
import dms.domain.document.ContentHash
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.document.DocumentMetadata
import dms.domain.library.RelativePath

/**
 * Catalogs files of the primary folder while a scan reads them, many in one catalog write. The files
 * keep their name and place; nothing on disk changes except the catalog. Their date is guessed by
 * [DocumentDates]. A file whose path or content the catalog already knows is left to the change
 * detection, because it may be a moved document.
 */
class AddScannedFiles(
    private val store: CatalogStore,
    private val dates: DocumentDates,
    private val activity: ActivityRecorder,
    private val clock: Clock,
    private val ids: IdGenerator,
) {

    /** Returns how many of [files] were added. */
    suspend operator fun invoke(files: List<ScannedFile>): Int {
        val primary = store.current()?.folders?.primary
        if (files.isEmpty() || primary == null) {
            return 0
        }
        val metadata = files.associate { it.path to dates.metadataFor(folder = primary, path = it.path) }
        var added = emptyList<Document>()
        val outcome = store.commit { catalog ->
            var current = catalog
            added = newDocuments(catalog = catalog, files = files, metadata = metadata).filter { document ->
                val update = current.addDocument(document = document)
                if (update is CatalogUpdate.Applied) {
                    current = update.catalog
                }
                update is CatalogUpdate.Applied
            }
            if (added.isEmpty()) {
                CatalogUpdate.Rejected(reason = CatalogRejection.PATH_ALREADY_USED)
            } else {
                CatalogUpdate.Applied(catalog = current)
            }
        }
        if (outcome !is OperationOutcome.Done) {
            return 0
        }
        val subject = added.singleOrNull()?.path?.value ?: "${added.size} files found by the scan"
        activity.record(kind = ActivityKind.DOCUMENT_ADDED, subject = subject)
        return added.size
    }

    private fun newDocuments(
        catalog: Catalog,
        files: List<ScannedFile>,
        metadata: Map<RelativePath, DocumentMetadata>,
    ): List<Document> {
        val knownPaths = catalog.documents.values.map { it.path }.toMutableSet()
        val knownHashes: MutableSet<ContentHash> = catalog.documents.values.map { it.contentHash }.toMutableSet()
        val now = clock.now()
        return files
            .filter { knownPaths.add(element = it.path) && knownHashes.add(element = it.contentHash) }
            .map { file ->
                Document(
                    id = DocumentId(value = ids.newId()),
                    path = file.path,
                    contentHash = file.contentHash,
                    size = file.size,
                    addedAt = now,
                    metadata = metadata[file.path] ?: DocumentMetadata.EMPTY,
                )
            }
    }
}
