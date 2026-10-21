package dms.domain.catalog

/** The merged catalog plus every decision the merge had to make. */
data class CatalogMergeResult(
    val catalog: Catalog,
    val conflicts: List<CatalogMergeConflict>,
)
