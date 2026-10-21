package dms.domain.catalog

import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId

/**
 * Merges two copies of the same catalog, for example the copies of two library folders, or a
 * conflict copy a sync client created when two devices saved at the same time.
 *
 * There is no common ancestor to compare against, so an entry missing on one side cannot be told
 * apart from an entry deleted there. The merge therefore never deletes: it is the union of both
 * copies. Where both copies hold different versions of the same entry, the copy with the higher
 * revision wins (the first one on a tie) and the decision is reported as a conflict.
 */
object CatalogMerger {

    fun merge(first: Catalog, second: Catalog): CatalogMergeResult {
        require(value = first.libraryId == second.libraryId) { "Only copies of the same library can be merged" }
        val newer = if (second.revision > first.revision) second else first
        val older = if (newer === first) second else first

        val categoryMerge = mergeCategories(newer = newer.categories, older = older.categories)
        val documentMerge = mergeDocuments(newer = newer.documents, older = older.documents)
        val merged = newer.copy(
            documents = documentMerge.entries,
            categories = categoryMerge.entries,
            tags = newer.tags + older.tags,
        )
        val conflicts = categoryMerge.conflicts + documentMerge.conflicts
        if (merged == newer) {
            return CatalogMergeResult(catalog = newer, conflicts = conflicts)
        }
        return CatalogMergeResult(catalog = merged.copy(revision = newer.revision.next()), conflicts = conflicts)
    }

    // Categories only in the older copy cannot form a cycle with the newer copy's tree: nothing in the
    // newer copy can point at an id it does not know.
    private fun mergeCategories(
        newer: Map<CategoryId, Category>,
        older: Map<CategoryId, Category>,
    ): PartialMerge<CategoryId, Category> {
        val conflicts = mutableListOf<CatalogMergeConflict>()
        val entries = newer.toMutableMap()
        older.values.forEach { category ->
            val kept = entries[category.id]
            when {
                kept == null -> entries[category.id] = category
                kept != category -> conflicts += CatalogMergeConflict.CategoryDiffers(kept = kept, discarded = category)
            }
        }
        return PartialMerge(entries = entries, conflicts = conflicts)
    }

    private fun mergeDocuments(
        newer: Map<DocumentId, Document>,
        older: Map<DocumentId, Document>,
    ): PartialMerge<DocumentId, Document> {
        val conflicts = mutableListOf<CatalogMergeConflict>()
        val entries = newer.toMutableMap()
        val ownerByPath = newer.values.associateBy { it.path }.toMutableMap()
        older.values.forEach { document ->
            val sameId = entries[document.id]
            val samePath = ownerByPath[document.path]
            when {
                sameId != null && sameId != document ->
                    conflicts += CatalogMergeConflict.DocumentDiffers(kept = sameId, discarded = document)

                sameId == null && samePath != null ->
                    conflicts += CatalogMergeConflict.PathClash(kept = samePath, discarded = document)

                sameId == null -> {
                    entries[document.id] = document
                    ownerByPath[document.path] = document
                }
            }
        }
        return PartialMerge(entries = entries, conflicts = conflicts)
    }

    private class PartialMerge<K, V>(
        val entries: Map<K, V>,
        val conflicts: List<CatalogMergeConflict>,
    )
}
