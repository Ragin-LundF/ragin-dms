package dms.domain.taxonomy.model

/** Identifies a category independently of its (renamable) name. */
@JvmInline
value class CategoryIdModel(val value: String) {
    init {
        require(value = value.isNotBlank()) { "Category id must not be blank" }
    }

    override fun toString(): String {
        return value
    }
}
