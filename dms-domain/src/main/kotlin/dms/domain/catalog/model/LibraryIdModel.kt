package dms.domain.catalog.model

/** Identifies a library. Every folder holding the same library carries the same id in its catalog. */
@JvmInline
value class LibraryIdModel(val value: String) {
    init {
        require(value = value.isNotBlank()) { "Library id must not be blank" }
    }

    override fun toString(): String {
        return value
    }
}
