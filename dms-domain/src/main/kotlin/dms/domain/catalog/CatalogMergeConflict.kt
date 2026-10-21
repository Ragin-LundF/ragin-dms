package dms.domain.catalog

import dms.domain.document.Document
import dms.domain.taxonomy.Category

/** Something [CatalogMerger] had to decide between two catalog copies. The user reviews each one. */
sealed interface CatalogMergeConflict {

    /** The same document differs in both copies; the newer copy's version was kept. */
    data class DocumentDiffers(val kept: Document, val discarded: Document) : CatalogMergeConflict

    /** Two different documents claim the same path; the newer copy's document was kept. */
    data class PathClash(val kept: Document, val discarded: Document) : CatalogMergeConflict

    /** The same category differs in both copies; the newer copy's version was kept. */
    data class CategoryDiffers(val kept: Category, val discarded: Category) : CatalogMergeConflict
}
