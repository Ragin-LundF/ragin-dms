package dms.application.common.model

import dms.domain.catalog.model.CatalogModel
import dms.domain.library.model.LibraryFoldersModel

/** The folders and the catalog of the open library at one moment. */
data class LibrarySnapshotModel(
    val folders: LibraryFoldersModel,
    val catalog: CatalogModel,
)
