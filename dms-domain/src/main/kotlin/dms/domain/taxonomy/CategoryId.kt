package dms.domain.taxonomy

/** Identifies a category independently of its (renamable) name. */
@JvmInline
value class CategoryId(val value: String) {
    init {
        require(value = value.isNotBlank()) { "Category id must not be blank" }
    }

    override fun toString(): String {
        return value
    }
}
