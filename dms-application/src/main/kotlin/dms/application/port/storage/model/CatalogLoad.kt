package dms.application.port.storage.model

import dms.domain.catalog.model.CatalogModel

/** The result of reading the catalog file of one library folder. */
sealed interface CatalogLoad {

    /** The catalog, plus any conflict copies a sync client created next to it. */
    data class Found(val catalog: CatalogModel, val conflictCopies: List<CatalogModel> = emptyList()) : CatalogLoad

    /** The folder has no catalog file yet. */
    data object NotFound : CatalogLoad

    data class Failed(val failure: StorageResult.Failure) : CatalogLoad
}
