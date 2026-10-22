package dms.domain.taxonomy.model

/**
 * What kind of document something is (invoice, contract, certificate, …). Categories form a tree
 * through [parentId]; a document has at most one category.
 *
 * [retentionYears] is an opt-in retention period the user set for documents of this category and
 * its subcategories; nothing suggests one.
 */
data class CategoryModel(
    val id: CategoryIdModel,
    val name: String,
    val parentId: CategoryIdModel? = null,
    val retentionYears: Int? = null,
) {
    init {
        require(value = name.isNotBlank()) { "Category name must not be blank" }
        require(value = parentId != id) { "A category cannot be its own parent" }
        require(value = retentionYears == null || retentionYears in 1..MAX_RETENTION_YEARS) {
            "Retention must be between 1 and $MAX_RETENTION_YEARS years"
        }
    }

    companion object {
        const val MAX_RETENTION_YEARS: Int = 100
    }
}
