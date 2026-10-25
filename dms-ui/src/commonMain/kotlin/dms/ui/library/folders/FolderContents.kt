package dms.ui.library.folders

import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.ui.library.LibraryTrees
import dms.ui.library.folders.model.FolderCrumbModel
import dms.ui.library.folders.model.FolderTileModel
import java.util.Locale

/** What a folder holds, derived from the catalog's documents and the folders that exist on disk. */
object FolderContents {

    const val NEW_FOLDER_NAME: String = "New folder"
    const val LIBRARY_NAME: String = "Library"

    /**
     * Every folder of the library, without the top level, in tree order: the folders of the catalog's
     * documents and the [foldersOnDisk], also empty ones. Tree, tiles, pickers and new names use this.
     */
    fun knownFolders(
        documents: Collection<DocumentModel>,
        foldersOnDisk: Set<RelativePathModel>,
    ): List<RelativePathModel> {
        return LibraryTrees.folders(documents = documents, foldersOnDisk = foldersOnDisk)
            .map { it.path }
            .filterNot { it.isRoot }
    }

    fun tiles(
        open: RelativePathModel,
        folders: List<RelativePathModel>,
        documents: Collection<DocumentModel>,
    ): List<FolderTileModel> {
        return childrenOf(parent = open, folders = folders).map { folder ->
            FolderTileModel(
                path = folder,
                name = folder.fileName,
                documentCount = documents.count { it.path.isInside(folder = folder) },
                folderCount = childrenOf(parent = folder, folders = folders).size,
            )
        }
    }

    /** "Library" first, then every folder down to [open]. */
    fun crumbs(open: RelativePathModel): List<FolderCrumbModel> {
        val path = generateSequence(seed = open) { it.parent }.filterNot { it.isRoot }.toList().asReversed()
        val library = FolderCrumbModel(path = RelativePathModel.ROOT, name = LIBRARY_NAME)
        return listOf(library) + path.map { FolderCrumbModel(path = it, name = it.fileName) }
    }

    /** The documents directly in [open] whose title or file name contains [query], newest first. */
    fun documentsIn(open: RelativePathModel, documents: Collection<DocumentModel>, query: String): List<DocumentModel> {
        val needle = query.trim().lowercase(Locale.ROOT)
        return documents
            .filter { (it.path.parent ?: RelativePathModel.ROOT) == open }
            .filter { needle.isEmpty() || matches(document = it, needle = needle) }
            .sortedWith(
                comparator = compareByDescending<DocumentModel> { it.metadata.documentDate }
                    .thenBy { it.path.fileName.lowercase(Locale.ROOT) },
            )
    }

    /** "New folder", or "New folder 2" and so on while a folder in [parent] already has that name. */
    fun proposedName(parent: RelativePathModel, folders: List<RelativePathModel>): String {
        val taken = childrenOf(parent = parent, folders = folders).map { it.fileName.lowercase(Locale.ROOT) }.toSet()
        return generateSequence(seed = 1) { it + 1 }
            .map { number -> if (number == 1) NEW_FOLDER_NAME else "$NEW_FOLDER_NAME $number" }
            .first { it.lowercase(Locale.ROOT) !in taken }
    }

    /** [paths] with everything at or below [from] moved below [to]. */
    fun relocated(
        paths: Set<RelativePathModel>,
        from: RelativePathModel,
        to: RelativePathModel,
    ): Set<RelativePathModel> {
        return paths.map { relocated(path = it, from = from, to = to) }.toSet()
    }

    /** [path] moved below [to] if it lies at or below [from], otherwise [path] itself. */
    fun relocated(path: RelativePathModel, from: RelativePathModel, to: RelativePathModel): RelativePathModel {
        return if (isAtOrBelow(path = path, folder = from)) path.relocate(from = from, to = to) else path
    }

    fun isAtOrBelow(path: RelativePathModel, folder: RelativePathModel): Boolean {
        return path == folder || path.isInside(folder = folder)
    }

    fun childrenOf(parent: RelativePathModel, folders: List<RelativePathModel>): List<RelativePathModel> {
        return folders.filter { (it.parent ?: RelativePathModel.ROOT) == parent }
    }

    private fun matches(document: DocumentModel, needle: String): Boolean {
        val title = document.metadata.title?.lowercase(Locale.ROOT).orEmpty()
        return needle in title || needle in document.path.fileName.lowercase(Locale.ROOT)
    }
}
