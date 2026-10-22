package dms.domain.link.model

import dms.domain.document.model.DocumentIdModel

/**
 * Two documents that belong together, such as an invoice and its reminder. A link has no direction:
 * [first] is always the smaller id, so the same pair is the same link however it was created.
 */
class DocumentLinkModel private constructor(
    val first: DocumentIdModel,
    val second: DocumentIdModel,
    val label: String?,
) {

    /** `true` if this link connects [documentId] with another document. */
    fun touches(documentId: DocumentIdModel): Boolean {
        return first == documentId || second == documentId
    }

    /** The other end of the link seen from [documentId]. */
    fun otherThan(documentId: DocumentIdModel): DocumentIdModel {
        return if (first == documentId) second else first
    }

    /** `true` if both links connect the same two documents, whatever their labels. */
    fun samePair(other: DocumentLinkModel): Boolean {
        return first == other.first && second == other.second
    }

    override fun equals(other: Any?): Boolean {
        return other is DocumentLinkModel && samePair(other = other) && label == other.label
    }

    override fun hashCode(): Int {
        return 31 * (31 * first.hashCode() + second.hashCode()) + label.hashCode()
    }

    override fun toString(): String {
        return "DocumentLink($first, $second, $label)"
    }

    companion object {

        fun of(first: DocumentIdModel, second: DocumentIdModel, label: String? = null): DocumentLinkModel {
            require(value = first != second) { "A document cannot be linked to itself" }
            require(value = label == null || label.isNotBlank()) { "Link label must be null or not blank" }
            val ordered = listOf(first, second).sortedBy { it.value }
            return DocumentLinkModel(first = ordered[0], second = ordered[1], label = label?.trim())
        }
    }
}
