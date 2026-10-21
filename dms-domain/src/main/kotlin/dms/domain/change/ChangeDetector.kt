package dms.domain.change

import dms.domain.catalog.Catalog
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath

/**
 * Compares the catalog with snapshots of the reachable library folders and reports every difference.
 *
 * Detection only: nothing here decides what to do about a change. Folders without a snapshot (for
 * example an offline network share) are not reported as missing anything.
 */
object ChangeDetector {

    fun detect(catalog: Catalog, snapshots: List<FolderSnapshot>): List<DetectedChange> {
        val knownPaths = catalog.documents.values.map { it.path }.toSet()
        val missingIn = mutableMapOf<DocumentId, MutableSet<LibraryFolderId>>()
        val presentIn = mutableMapOf<DocumentId, MutableSet<LibraryFolderId>>()
        val untracked = mutableMapOf<RelativePath, MutableMap<LibraryFolderId, FileFacts>>()
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
        document: Document,
        snapshot: FolderSnapshot,
        unclaimed: MutableMap<RelativePath, FileFacts>,
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

    private fun modification(document: Document, facts: FileFacts, folderId: LibraryFolderId): DetectedChange? {
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
        catalog: Catalog,
        missingIn: Map<DocumentId, Set<LibraryFolderId>>,
        presentIn: Map<DocumentId, Set<LibraryFolderId>>,
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

    private fun untrackedChange(path: RelativePath, factsByFolder: Map<LibraryFolderId, FileFacts>): DetectedChange {
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

    private fun MutableMap<DocumentId, MutableSet<LibraryFolderId>>.addFolder(
        documentId: DocumentId,
        folderId: LibraryFolderId,
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
