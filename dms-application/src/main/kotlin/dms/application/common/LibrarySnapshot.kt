package dms.application.common

import dms.domain.catalog.Catalog
import dms.domain.library.LibraryFolders

/** The folders and the catalog of the open library at one moment. */
data class LibrarySnapshot(
    val folders: LibraryFolders,
    val catalog: Catalog,
)
