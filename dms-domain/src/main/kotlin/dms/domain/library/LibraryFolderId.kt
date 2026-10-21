package dms.domain.library

/** Identifies one configured library folder. */
@JvmInline
value class LibraryFolderId(val value: String) {
    init {
        require(value = value.isNotBlank()) { "Library folder id must not be blank" }
    }

    override fun toString(): String {
        return value
    }
}
