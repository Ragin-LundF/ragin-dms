package dms.ui.library.folders.model

import dms.application.port.state.model.LayoutPreferencesModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.ui.library.model.FolderEntryModel

/**
 * What the folder view shows: the [open] folder with its [crumbs], the visible [tree] rows below the
 * "Library" row (which counts all [libraryCount] documents), the subfolder [tiles] and the [documents]
 * directly in the open folder that match [query]. [busyReason] says why its buttons wait.
 */
data class FolderBrowserStateModel(
    val open: RelativePathModel = RelativePathModel.ROOT,
    val crumbs: List<FolderCrumbModel> = emptyList(),
    val tree: List<FolderEntryModel> = emptyList(),
    val libraryCount: Int = 0,
    val tiles: List<FolderTileModel> = emptyList(),
    val documents: List<DocumentModel> = emptyList(),
    val naming: FolderNaming? = null,
    val query: String = "",
    val searchPlaceholder: String = "",
    val treeWidth: Int = LayoutPreferencesModel.DEFAULT_FOLDER_TREE_WIDTH,
    val treeHidden: Boolean = false,
    val busyReason: String? = null,
)
