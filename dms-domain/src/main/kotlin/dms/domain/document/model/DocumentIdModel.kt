package dms.domain.document.model

/**
 * Identifies a document for its whole life. Paths change when a file is moved or renamed; the id
 * does not.
 */
@JvmInline
value class DocumentIdModel(val value: String) {
    init {
        require(value = value.isNotBlank()) { "Document id must not be blank" }
    }

    override fun toString(): String {
        return value
    }
}
