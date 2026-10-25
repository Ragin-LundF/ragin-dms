package dms.ui.library.folders.model

import dms.domain.library.model.RelativePathModel

/**
 * What the user chose in the folder view: the [open] folder, the [expanded] tree rows, the name typed in
 * place ([naming], typed so far: [draft]) and the search text. Which folders exist comes from disk.
 */
data class FolderViewStateModel(
    val open: RelativePathModel = RelativePathModel.ROOT,
    val expanded: Set<RelativePathModel> = emptySet(),
    val naming: FolderNaming? = null,
    val draft: String = "",
    val query: String = "",
)
