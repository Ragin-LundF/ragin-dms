package dms.ui.settings.model

import dms.domain.library.LibraryFolder

/** One configured library folder; [note] says why the tool currently leaves it alone, if it does. */
data class FolderRow(
    val folder: LibraryFolder,
    val isPrimary: Boolean,
    val note: String?,
)
