package dms.application.resolution

import dms.domain.document.model.DocumentModel

/**
 * Which of several copies of a file to keep by default: the one the user did the most work on
 * (category, tags, AI classification, title), and among equals the one that was catalogued first.
 */
object DuplicatePreference {

    fun preferred(copies: Collection<DocumentModel>): DocumentModel {
        return copies.sortedWith(
            comparator = compareByDescending<DocumentModel> { it.categoryId != null }
                .thenByDescending { it.tags.size }
                .thenByDescending { it.classification != null }
                .thenByDescending { it.metadata.title != null }
                .thenBy { it.addedAt },
        ).first()
    }
}
