package dms.domain.catalog

import dms.domain.catalog.model.CatalogModel

/** The result of a change to a [CatalogModel]. */
sealed interface CatalogUpdate {

    /** Applies [next] to the resulting catalog; a rejection short-circuits and stays a rejection. */
    fun andThen(next: (CatalogModel) -> CatalogUpdate): CatalogUpdate {
        return when (this) {
            is Applied -> next(catalog)
            is Rejected -> this
        }
    }

    /** The change was applied; [catalog] carries the next revision. */
    data class Applied(val catalog: CatalogModel) : CatalogUpdate

    /** The change was refused; the original catalog is unchanged. */
    data class Rejected(val reason: CatalogRejection) : CatalogUpdate
}
