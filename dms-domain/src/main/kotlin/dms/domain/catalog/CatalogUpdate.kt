package dms.domain.catalog

/** The result of a change to a [Catalog]. */
sealed interface CatalogUpdate {

    /** Applies [next] to the resulting catalog; a rejection short-circuits and stays a rejection. */
    fun andThen(next: (Catalog) -> CatalogUpdate): CatalogUpdate {
        return when (this) {
            is Applied -> next(catalog)
            is Rejected -> this
        }
    }

    /** The change was applied; [catalog] carries the next revision. */
    data class Applied(val catalog: Catalog) : CatalogUpdate

    /** The change was refused; the original catalog is unchanged. */
    data class Rejected(val reason: CatalogRejection) : CatalogUpdate
}
