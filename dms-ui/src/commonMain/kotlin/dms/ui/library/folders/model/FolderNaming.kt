package dms.ui.library.folders.model

import dms.domain.library.model.RelativePathModel

/** A folder name being typed in place, as a tile of the open folder; nothing is written before it is saved. */
sealed interface FolderNaming {

    /** The folder the name goes into. */
    val parent: RelativePathModel

    /** A new folder in [parent], proposed as [proposed] ("New folder", "New folder 2", …). */
    data class New(override val parent: RelativePathModel, val proposed: String) : FolderNaming

    /** A new name for the folder at [path]. */
    data class Rename(val path: RelativePathModel) : FolderNaming {
        override val parent: RelativePathModel
            get() {
                return path.parent ?: RelativePathModel.ROOT
            }
    }
}
