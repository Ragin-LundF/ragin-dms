package dms.ui.library.model

import dms.domain.taxonomy.Category

/** One category of the category tree, flattened: [depth] is its indentation; subcategories show while [expanded]. */
data class CategoryEntry(
    val category: Category,
    val depth: Int,
    val documentCount: Int,
    val hasChildren: Boolean = false,
    val expanded: Boolean = false,
)
