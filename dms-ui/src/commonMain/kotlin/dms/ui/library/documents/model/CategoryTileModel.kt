package dms.ui.library.documents.model

import dms.domain.document.DocumentKind
import dms.domain.taxonomy.model.CategoryIdModel

/**
 * A top-level category as a tile above "All documents": its name, how many documents it holds with its
 * subcategories, and the [kinds] of up to three of them for the stacked page previews.
 */
data class CategoryTileModel(
    val categoryId: CategoryIdModel,
    val name: String,
    val count: Int,
    val kinds: List<DocumentKind>,
)
