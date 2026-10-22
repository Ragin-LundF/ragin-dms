package dms.domain.classification

import dms.domain.taxonomy.model.CategoryIdModel

/** The category an AI suggests: one the catalog already has, or a new one the user may create. */
sealed interface SuggestedCategory {

    data class Existing(val categoryId: CategoryIdModel) : SuggestedCategory

    data class New(val name: String, val parentId: CategoryIdModel?) : SuggestedCategory {
        init {
            require(value = name.isNotBlank()) { "Suggested category name must not be blank" }
        }
    }
}
