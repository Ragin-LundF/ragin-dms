package dms.domain.change

import dms.domain.catalog.model.CatalogModel
import dms.domain.change.model.FileFactsModel
import dms.domain.change.model.FolderSnapshotModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel

/**
 * Compares the catalog with snapshots of the reachable library folders and reports every difference.
 *
 * Detection only: nothing here decides what to do about a change. Folders without a snapshot (for
 * example an offline network share) are not reported as missing anything.
 */
object ChangeDetector {

    fun detect(catalog: CatalogModel, snapshots: List<FolderSnapshotModel>): List<DetectedChange> {
        val knownPaths = catalog.documents.values.map { it.path }.toSet()
        val missingIn = mutableMapOf<DocumentIdModel, MutableSet<LibraryFolderIdModel>>()
        val presentIn = mutableMapOf<DocumentIdModel, MutableSet<LibraryFolderIdModel>>()
        val untracked = mutableMapOf<RelativePathModel, MutableMap<LibraryFolderIdModel, FileFactsModel>>()
        val changes = mutableListOf<DetectedChange>()

        snapshots.forEach { snapshot ->
            val unclaimed = snapshot.files.filterKeys { it !in knownPaths }.toMutableMap()
            catalog.documents.values.forEach { document ->
                when (val comparison = compare(document = document, snapshot = snapshot, unclaimed = unclaimed)) {
                    Comparison.Unchanged -> presentIn.addFolder(documentId = document.id, folderId = snapshot.folderId)
                    Comparison.Missing -> missingIn.addFolder(documentId = document.id, folderId = snapshot.folderId)
                    is Comparison.Changed -> changes += comparison.change
                }
            }
            unclaimed.forEach { (path, facts) ->
                untracked.getOrPut(key = path) { mutableMapOf() }[snapshot.folderId] = facts
            }
        }

        changes += missingFiles(catalog = catalog, missingIn = missingIn, presentIn = presentIn)
        changes += untracked.map { (path, facts) -> untrackedChange(path = path, factsByFolder = facts) }
        return changes.sortedBy { it.path.value }
    }

    /** Compares one document with one folder. A move claims the untracked file it matched. */
    private fun compare(
        document: DocumentModel,
        snapshot: FolderSnapshotModel,
        unclaimed: MutableMap<RelativePathModel, FileFactsModel>,
    ): Comparison {
        val facts = snapshot.files[document.path]
        if (facts != null) {
            val modification = modification(document = document, facts = facts, folderId = snapshot.folderId)
            return if (modification == null) Comparison.Unchanged else Comparison.Changed(change = modification)
        }
        val newPath = unclaimed.entries.firstOrNull { it.value.contentHash == document.contentHash }?.key
        if (newPath != null) {
            unclaimed.remove(key = newPath)
            val move = DetectedChange.MovedFile(
                documentId = document.id,
                path = newPath,
                previousPath = document.path,
                folderId = snapshot.folderId,
            )
            return Comparison.Changed(change = move)
        }
        return Comparison.Missing
    }

    private fun modification(
        document: DocumentModel,
        facts: FileFactsModel,
        folderId: LibraryFolderIdModel,
    ): DetectedChange? {
        val changed = if (facts.contentHash != null) {
            facts.contentHash != document.contentHash
        } else {
            facts.size != document.size
        }
        if (!changed) {
            return null
        }
        return DetectedChange.ModifiedFile(
            documentId = document.id,
            path = document.path,
            folderId = folderId,
            size = facts.size,
            contentHash = facts.contentHash,
        )
    }

    private fun missingFiles(
        catalog: CatalogModel,
        missingIn: Map<DocumentIdModel, Set<LibraryFolderIdModel>>,
        presentIn: Map<DocumentIdModel, Set<LibraryFolderIdModel>>,
    ): List<DetectedChange> {
        return missingIn.map { (documentId, folders) ->
            DetectedChange.MissingFile(
                documentId = documentId,
                path = catalog.documents.getValue(key = documentId).path,
                missingIn = folders,
                presentIn = presentIn[documentId].orEmpty(),
            )
        }
    }

    private fun untrackedChange(
        path: RelativePathModel,
        factsByFolder: Map<LibraryFolderIdModel,
        FileFactsModel>,
    ): DetectedChange {
        val hashes = factsByFolder.mapNotNull { (folderId, facts) -> facts.contentHash?.let { folderId to it } }.toMap()
        if (hashes.values.toSet().size > 1) {
            return DetectedChange.ContentConflict(path = path, contentHashes = hashes)
        }
        val facts = factsByFolder.values.first()
        return DetectedChange.NewFile(
            path = path,
            presentIn = factsByFolder.keys,
            size = facts.size,
            contentHash = hashes.values.firstOrNull(),
        )
    }

    private fun MutableMap<DocumentIdModel, MutableSet<LibraryFolderIdModel>>.addFolder(
        documentId: DocumentIdModel,
        folderId: LibraryFolderIdModel,
    ) {
        getOrPut(key = documentId) { mutableSetOf() }.add(element = folderId)
    }

    /** How one document relates to one folder. */
    private sealed interface Comparison {
        data object Unchanged : Comparison

        data object Missing : Comparison

        data class Changed(val change: DetectedChange) : Comparison
    }
}
