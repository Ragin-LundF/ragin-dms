package dms.domain.query.model

/** Identifies a saved search independently of its (renamable) name. */
@JvmInline
value class SavedSearchIdModel(val value: String) {
    init {
        require(value = value.isNotBlank()) { "Saved search id must not be blank" }
    }

    override fun toString(): String {
        return value
    }
}
