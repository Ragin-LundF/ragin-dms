package dms.application.resolution

import dms.domain.document.Document

/**
 * Which of several copies of a file to keep by default: the one the user did the most work on
 * (category, tags, AI classification, title), and among equals the one that was catalogued first.
 */
object DuplicatePreference {

    fun preferred(copies: Collection<Document>): Document {
        return copies.sortedWith(
            comparator = compareByDescending<Document> { it.categoryId != null }
                .thenByDescending { it.tags.size }
                .thenByDescending { it.classification != null }
                .thenByDescending { it.metadata.title != null }
                .thenBy { it.addedAt },
        ).first()
    }
}
