package dms.ui.library.picker.model

import dms.domain.library.model.RelativePathModel

/**
 * An open "Move to…" picker: its [title] and [confirmLabel], the [selected] folder (the library's top
 * level is [RelativePathModel.ROOT]), the [expanded] folders of the tree, and [excluded], a folder that
 * cannot be the target together with everything below it, such as the folder being moved.
 */
data class FolderPickerStateModel(
    val title: String,
    val confirmLabel: String,
    val selected: RelativePathModel? = null,
    val expanded: Set<RelativePathModel> = emptySet(),
    val excluded: RelativePathModel? = null,
) {
    fun isExcluded(path: RelativePathModel): Boolean {
        val folder = excluded ?: return false
        return path == folder || path.isInside(folder = folder)
    }
}
