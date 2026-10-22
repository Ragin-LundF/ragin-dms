package dms.ui.dashboard.task

import dms.domain.catalog.model.CatalogModel
import dms.domain.change.DetectedChange
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFoldersModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel

/** Names of folders, documents and categories for the texts of tasks. */
class TaskContext(
    private val folders: LibraryFoldersModel?,
    private val catalog: CatalogModel?,
) {

    val primaryId: LibraryFolderIdModel?
        get() {
            return folders?.primary?.id
        }

    fun folderName(id: LibraryFolderIdModel): String {
        return folders?.find(id = id)?.displayName ?: id.value
    }

    fun folderNames(ids: Collection<LibraryFolderIdModel>): String {
        if (ids.isEmpty()) {
            return "no reachable folder"
        }
        return ids.joinToString(separator = ", ") { folderName(id = it) }
    }

    fun documentPath(id: DocumentIdModel): String {
        return catalog?.documents?.let { it[id] }?.path?.value ?: id.value
    }

    /** The catalog path of a document, or `null` if the catalog does not know it. */
    fun pathOf(id: DocumentIdModel): RelativePathModel? {
        return catalog?.documents?.let { it[id] }?.path
    }

    /** [ids] with the primary folder first. */
    fun primaryFirst(ids: Collection<LibraryFolderIdModel>): List<LibraryFolderIdModel> {
        return ids.sortedBy { it != primaryId }
    }

    /**
     * The catalogued document [change] is a copy of, by size and SHA-256, or `null` if there is none.
     * A file at a path the catalog already knows is that document itself, not a copy.
     */
    fun originalOf(change: DetectedChange.NewFile): DocumentModel? {
        val contentHash = change.contentHash ?: return null
        if (catalog?.findByPath(path = change.path) != null) {
            return null
        }
        return catalog?.findByContentHash(contentHash = contentHash)?.firstOrNull { it.size == change.size }
    }

    /** The catalogued documents of [ids]; ids the catalog no longer knows are left out. */
    fun documents(ids: List<DocumentIdModel>): List<DocumentModel> {
        return ids.mapNotNull { id -> catalog?.documents?.let { it[id] } }
    }

    fun fieldLabel(key: FieldKeyModel): String {
        return catalog?.settings?.field(key = key)?.label ?: key.value
    }

    fun categoryName(id: CategoryIdModel): String {
        return catalog?.categories?.let { it[id] }?.name ?: id.value
    }

    /** A folder other than [folderId], the primary first, or `null` if there is none. */
    fun otherFolderThan(folderId: LibraryFolderIdModel): LibraryFolderIdModel? {
        return primaryFirst(ids = folders?.folders.orEmpty().map { it.id }).firstOrNull { it != folderId }
    }
}
