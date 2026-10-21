package dms.domain.taxonomy

/**
 * What kind of document something is (invoice, contract, certificate, …). Categories form a tree
 * through [parentId]; a document has at most one category.
 */
data class Category(
    val id: CategoryId,
    val name: String,
    val parentId: CategoryId? = null,
) {
    init {
        require(value = name.isNotBlank()) { "Category name must not be blank" }
        require(value = parentId != id) { "A category cannot be its own parent" }
    }
}
