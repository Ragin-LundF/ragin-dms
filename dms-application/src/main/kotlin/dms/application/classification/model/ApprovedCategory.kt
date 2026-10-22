package dms.application.classification.model

import dms.domain.taxonomy.model.CategoryIdModel

/** The category the user approved: one the catalog has, or a path of names from the top level. */
sealed interface ApprovedCategory {

    data class Existing(val categoryId: CategoryIdModel) : ApprovedCategory

    /**
     * A category by its [names] from the top level down, like `Insurance / Household / Rent`. A level
     * whose name already exists at the same place is reused; every other level is created.
     */
    data class Path(val names: List<String>) : ApprovedCategory {
        init {
            require(value = names.isNotEmpty()) { "A category path needs at least one name" }
            require(value = names.none { it.isBlank() }) { "A category name must not be blank" }
        }
    }
}
