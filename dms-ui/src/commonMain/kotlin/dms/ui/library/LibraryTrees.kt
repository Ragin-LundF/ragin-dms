package dms.ui.library

import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.ui.library.model.CategoryEntryModel
import dms.ui.library.model.FolderEntryModel
import java.util.Locale

/** Builds the folder tree and the category tree of the sidebar as indented lists. */
object LibraryTrees {

    /**
     * The root first, then every folder in path order, each followed by its subfolders while it is in
     * [expanded]; `null` unfolds everything. Top-level folders always show.
     */
    fun folders(
        documents: Collection<DocumentModel>,
        createdFolders: Set<RelativePathModel>,
        expanded: Set<RelativePathModel>? = null,
    ): List<FolderEntryModel> {
        val withDocuments = documents.flatMap { ancestorsOf(path = it.path) }
        val created = createdFolders.flatMap { selfAndAncestors(folder = it) }
        // Sorting by segments keeps every folder directly above its subfolders.
        val folders = (withDocuments + created).toSet().sortedWith(
            comparator = compareBy { path -> path.segments.joinToString(separator = "\u0000").lowercase(Locale.ROOT) },
        )
        val root = FolderEntryModel(
            path = RelativePathModel.ROOT,
            name = "All documents",
            depth = 0,
            documentCount = documents.size,
            hasChildren = folders.isNotEmpty(),
            expanded = true,
        )
        val entries = listOf(root) + folders.mapIndexed { index, folder ->
            FolderEntryModel(
                path = folder,
                name = folder.fileName,
                depth = folder.segments.size,
                documentCount = documents.count { it.path.isInside(folder = folder) },
                hasChildren = folders.getOrNull(index = index + 1)?.isInside(folder = folder) == true,
                expanded = expanded == null || folder in expanded,
            )
        }
        return visible(entries = entries, depthOf = { it.depth }, folded = { it.hasChildren && !it.expanded })
    }

    /** Top-level categories by name, each followed by its subcategories while in [expanded]; `null` unfolds all. */
    fun categories(
        categories: Collection<CategoryModel>,
        documents: Collection<DocumentModel>,
        expanded: Set<CategoryIdModel>? = null,
    ): List<CategoryEntryModel> {
        val children = categories.groupBy { it.parentId }
        val counts = documents.groupingBy { it.categoryId }.eachCount()
        val entries = flatten(parentId = null, depth = 0, children = children, counts = counts).map { entry ->
            entry.copy(
                hasChildren = children[entry.category.id].orEmpty().isNotEmpty(),
                expanded = expanded == null || entry.category.id in expanded,
            )
        }
        return visible(entries = entries, depthOf = { it.depth }, folded = { it.hasChildren && !it.expanded })
    }

    /** Drops every entry below a [folded] one; entries are in tree order, children right after their parent. */
    private fun <T> visible(entries: List<T>, depthOf: (T) -> Int, folded: (T) -> Boolean): List<T> {
        var hiddenBelow = Int.MAX_VALUE
        return entries.filter { entry ->
            val depth = depthOf(entry)
            if (depth > hiddenBelow) {
                return@filter false
            }
            hiddenBelow = if (folded(entry)) depth else Int.MAX_VALUE
            true
        }
    }

    private fun flatten(
        parentId: CategoryIdModel?,
        depth: Int,
        children: Map<CategoryIdModel?, List<CategoryModel>>,
        counts: Map<CategoryIdModel?, Int>,
    ): List<CategoryEntryModel> {
        return children[parentId].orEmpty()
            .sortedBy { it.name.lowercase(Locale.ROOT) }
            .flatMap { category ->
                val entry = CategoryEntryModel(
                    category = category,
                    depth = depth,
                    documentCount = counts[category.id] ?: 0,
                )
                listOf(entry) + flatten(parentId = category.id, depth = depth + 1, children = children, counts = counts)
            }
    }

    /** The folders containing [path], without the root. */
    private fun ancestorsOf(path: RelativePathModel): List<RelativePathModel> {
        return path.parent?.let { selfAndAncestors(folder = it) }.orEmpty()
    }

    private fun selfAndAncestors(folder: RelativePathModel): List<RelativePathModel> {
        if (folder.isRoot) {
            return emptyList()
        }
        return listOf(folder) + ancestorsOf(path = folder)
    }
}
