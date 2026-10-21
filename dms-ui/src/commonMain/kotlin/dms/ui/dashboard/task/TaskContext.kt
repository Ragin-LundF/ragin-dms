package dms.ui.dashboard.task

import dms.domain.catalog.Catalog
import dms.domain.document.DocumentId
import dms.domain.library.LibraryFolderId
import dms.domain.library.LibraryFolders
import dms.domain.taxonomy.CategoryId

/** Names of folders, documents and categories for the texts of tasks. */
class TaskContext(
    private val folders: LibraryFolders?,
    private val catalog: Catalog?,
) {

    val primaryId: LibraryFolderId?
        get() {
            return folders?.primary?.id
        }

    fun folderName(id: LibraryFolderId): String {
        return folders?.find(id = id)?.displayName ?: id.value
    }

    fun folderNames(ids: Collection<LibraryFolderId>): String {
        if (ids.isEmpty()) {
            return "no reachable folder"
        }
        return ids.joinToString(separator = ", ") { folderName(id = it) }
    }

    fun documentPath(id: DocumentId): String {
        return catalog?.documents?.let { it[id] }?.path?.value ?: id.value
    }

    fun categoryName(id: CategoryId): String {
        return catalog?.categories?.let { it[id] }?.name ?: id.value
    }

    /** A folder other than [folderId], the primary first, or `null` if there is none. */
    fun otherFolderThan(folderId: LibraryFolderId): LibraryFolderId? {
        val all = folders?.folders.orEmpty().sortedBy { it.id != primaryId }
        return all.firstOrNull { it.id != folderId }?.id
    }
}
