package dms.domain.library.model

/** Identifies one configured library folder. */
@JvmInline
value class LibraryFolderIdModel(val value: String) {
    init {
        require(value = value.isNotBlank()) { "Library folder id must not be blank" }
    }

    override fun toString(): String {
        return value
    }
}
