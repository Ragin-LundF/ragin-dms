package dms.domain.catalog

/** Identifies a library. Every folder holding the same library carries the same id in its catalog. */
@JvmInline
value class LibraryId(val value: String) {
    init {
        require(value = value.isNotBlank()) { "Library id must not be blank" }
    }

    override fun toString(): String {
        return value
    }
}
