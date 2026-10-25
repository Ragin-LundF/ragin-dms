package dms.ui.library.folders

import dms.application.port.storage.model.StorageFailure
import dms.domain.library.model.RelativePathModel
import dms.ui.library.folders.model.FolderTileModel
import dms.ui.session.OutcomeMessages

/** The words of the folder view. */
object FolderTexts {

    fun deleteTitle(folder: RelativePathModel): String {
        return "Delete \"${folder.fileName}\"?"
    }

    /**
     * What deleting a folder takes along: its [documents], the [otherFiles] the library does not know (new
     * files, or files only in a sync folder) and its [subfolders]. "Empty" only when nothing is on disk.
     */
    fun deleteMessage(documents: Int, otherFiles: Int, subfolders: Int): String {
        if (documents == 0 && otherFiles == 0 && subfolders == 0) {
            return "The folder is empty. It is removed from every library folder."
        }
        val parts = listOfNotNull(
            documents.takeIf { it > 0 }?.let { counted(count = it, singular = "document") },
            otherFiles.takeIf { it > 0 }?.let { "${counted(count = it, singular = "other file")} not in the library" },
            subfolders.takeIf { it > 0 }?.let { counted(count = it, singular = "subfolder") },
        )
        val verb = if (documents + otherFiles + subfolders == 1) "goes" else "go"
        val listed = when (parts.size) {
            1 -> parts.single()
            else -> parts.dropLast(n = 1).joinToString(separator = ", ") + " and " + parts.last()
        }
        return "$listed inside $verb to the trash, in every library folder."
    }

    /** "2 documents · 1 folder", or "Empty". */
    fun tileCaption(tile: FolderTileModel): String {
        val parts = listOfNotNull(
            tile.documentCount.takeIf { it > 0 }?.let { counted(count = it, singular = "document") },
            tile.folderCount.takeIf { it > 0 }?.let { counted(count = it, singular = "folder") },
        )
        return parts.joinToString(separator = " · ").ifEmpty { "Empty" }
    }

    /** Why a folder cannot be deleted now: what is inside could not be read, so it cannot be named. */
    fun unreadable(folder: RelativePathModel, reason: StorageFailure): String {
        return "\"${folder.fileName}\" cannot be deleted now: its contents could not be read " +
            "(${OutcomeMessages.failure(reason = reason)})"
    }

    fun searchPlaceholder(open: RelativePathModel): String {
        return "Search in ${quotedOrTopLevel(folder = open)}…"
    }

    fun emptyDocuments(open: RelativePathModel): String {
        return "No documents directly in ${quotedOrTopLevel(folder = open)}. Drop files here to add them."
    }

    fun where(folder: RelativePathModel): String {
        return folder.value.ifEmpty { "the top level" }
    }

    private fun quotedOrTopLevel(folder: RelativePathModel): String {
        return if (folder.isRoot) "the top level" else "\"${folder.fileName}\""
    }

    private fun counted(count: Int, singular: String): String {
        return if (count == 1) "1 $singular" else "$count ${singular}s"
    }
}
