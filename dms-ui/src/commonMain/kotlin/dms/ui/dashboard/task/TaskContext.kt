package dms.ui.dashboard.task

import dms.domain.catalog.Catalog
import dms.domain.change.DetectedChange
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.library.LibraryFolderId
import dms.domain.library.LibraryFolders
import dms.domain.library.RelativePath
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

    /** The catalog path of a document, or `null` if the catalog does not know it. */
    fun pathOf(id: DocumentId): RelativePath? {
        return catalog?.documents?.let { it[id] }?.path
    }

    /** [ids] with the primary folder first. */
    fun primaryFirst(ids: Collection<LibraryFolderId>): List<LibraryFolderId> {
        return ids.sortedBy { it != primaryId }
    }

    /** The catalogued document [change] is a copy of, by size and SHA-256, or `null` if there is none. */
    fun originalOf(change: DetectedChange.NewFile): Document? {
        val contentHash = change.contentHash ?: return null
        return catalog?.findByContentHash(contentHash = contentHash)?.firstOrNull { it.size == change.size }
    }

    /** The catalogued documents of [ids]; ids the catalog no longer knows are left out. */
    fun documents(ids: List<DocumentId>): List<Document> {
        return ids.mapNotNull { id -> catalog?.documents?.let { it[id] } }
    }

    fun categoryName(id: CategoryId): String {
        return catalog?.categories?.let { it[id] }?.name ?: id.value
    }

    /** A folder other than [folderId], the primary first, or `null` if there is none. */
    fun otherFolderThan(folderId: LibraryFolderId): LibraryFolderId? {
        return primaryFirst(ids = folders?.folders.orEmpty().map { it.id }).firstOrNull { it != folderId }
    }
}
