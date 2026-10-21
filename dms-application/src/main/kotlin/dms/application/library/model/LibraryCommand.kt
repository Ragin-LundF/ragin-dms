package dms.application.library.model

import dms.domain.library.FolderLocation
import dms.domain.library.LibraryFolderId

/** A change to the library folders of this device. */
sealed interface LibraryCommand {

    /** The first configuration: the folder that becomes the primary. */
    data class CreatePrimary(val displayName: String, val location: FolderLocation) : LibraryCommand

    data class AddSyncFolder(val displayName: String, val location: FolderLocation) : LibraryCommand

    data class RemoveSyncFolder(val folderId: LibraryFolderId) : LibraryCommand

    /** Makes another folder the primary; the old primary becomes a sync folder. */
    data class ChoosePrimary(val folderId: LibraryFolderId) : LibraryCommand
}
