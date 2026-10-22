package dms.ui.library.model

import dms.domain.taxonomy.model.CategoryModel

/** One category of the category tree, flattened: [depth] is its indentation; subcategories show while [expanded]. */
data class CategoryEntryModel(
    val category: CategoryModel,
    val depth: Int,
    val documentCount: Int,
    val hasChildren: Boolean = false,
    val expanded: Boolean = false,
)
