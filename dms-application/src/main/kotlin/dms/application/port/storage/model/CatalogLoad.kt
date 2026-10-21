package dms.application.port.storage.model

import dms.domain.catalog.Catalog

/** The result of reading the catalog file of one library folder. */
sealed interface CatalogLoad {

    /** The catalog, plus any conflict copies a sync client created next to it. */
    data class Found(val catalog: Catalog, val conflictCopies: List<Catalog> = emptyList()) : CatalogLoad

    /** The folder has no catalog file yet. */
    data object NotFound : CatalogLoad

    data class Failed(val failure: StorageResult.Failure) : CatalogLoad
}
