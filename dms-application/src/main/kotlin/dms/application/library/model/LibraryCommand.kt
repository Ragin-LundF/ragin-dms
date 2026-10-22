package dms.application.library.model

import dms.domain.library.model.FolderLocationModel
import dms.domain.library.model.LibraryFolderIdModel

/** A change to the library folders of this device. */
sealed interface LibraryCommand {

    /** The first configuration: the folder that becomes the primary. */
    data class CreatePrimary(val displayName: String, val location: FolderLocationModel) : LibraryCommand

    data class AddSyncFolder(val displayName: String, val location: FolderLocationModel) : LibraryCommand

    data class RemoveSyncFolder(val folderId: LibraryFolderIdModel) : LibraryCommand

    /** Makes another folder the primary; the old primary becomes a sync folder. */
    data class ChoosePrimary(val folderId: LibraryFolderIdModel) : LibraryCommand
}
