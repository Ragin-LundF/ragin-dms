package dms.domain.catalog.model

import dms.domain.catalog.CatalogMergeConflict

/** The merged catalog plus every decision the merge had to make. */
data class CatalogMergeResultModel(
    val catalog: CatalogModel,
    val conflicts: List<CatalogMergeConflict>,
)
