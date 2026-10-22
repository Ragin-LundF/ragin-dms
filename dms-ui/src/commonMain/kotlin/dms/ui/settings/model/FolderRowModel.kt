package dms.ui.settings.model

import dms.domain.library.model.LibraryFolderModel

/** One configured library folder; [note] says why the tool currently leaves it alone, if it does. */
data class FolderRowModel(
    val folder: LibraryFolderModel,
    val isPrimary: Boolean,
    val note: String?,
)
