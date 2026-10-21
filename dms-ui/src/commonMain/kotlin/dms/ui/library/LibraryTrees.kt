package dms.ui.library

import dms.domain.document.Document
import dms.domain.library.RelativePath
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId
import dms.ui.library.model.CategoryEntry
import dms.ui.library.model.FolderEntry
import java.util.Locale

/** Builds the folder tree and the category tree of the sidebar as indented lists. */
object LibraryTrees {

    /**
     * The root first, then every folder in path order, each followed by its subfolders while it is in
     * [expanded]; `null` unfolds everything. Top-level folders always show.
     */
    fun folders(
        documents: Collection<Document>,
        createdFolders: Set<RelativePath>,
        expanded: Set<RelativePath>? = null,
    ): List<FolderEntry> {
        val withDocuments = documents.flatMap { ancestorsOf(path = it.path) }
        val created = createdFolders.flatMap { selfAndAncestors(folder = it) }
        // Sorting by segments keeps every folder directly above its subfolders.
        val folders = (withDocuments + created).toSet().sortedWith(
            comparator = compareBy { path -> path.segments.joinToString(separator = "\u0000").lowercase(Locale.ROOT) },
        )
        val root = FolderEntry(
            path = RelativePath.ROOT,
            name = "All documents",
            depth = 0,
            documentCount = documents.size,
            hasChildren = folders.isNotEmpty(),
            expanded = true,
        )
        val entries = listOf(root) + folders.mapIndexed { index, folder ->
            FolderEntry(
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
        categories: Collection<Category>,
        documents: Collection<Document>,
        expanded: Set<CategoryId>? = null,
    ): List<CategoryEntry> {
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
        parentId: CategoryId?,
        depth: Int,
        children: Map<CategoryId?, List<Category>>,
        counts: Map<CategoryId?, Int>,
    ): List<CategoryEntry> {
        return children[parentId].orEmpty()
            .sortedBy { it.name.lowercase(Locale.ROOT) }
            .flatMap { category ->
                val entry = CategoryEntry(category = category, depth = depth, documentCount = counts[category.id] ?: 0)
                listOf(entry) + flatten(parentId = category.id, depth = depth + 1, children = children, counts = counts)
            }
    }

    /** The folders containing [path], without the root. */
    private fun ancestorsOf(path: RelativePath): List<RelativePath> {
        return path.parent?.let { selfAndAncestors(folder = it) }.orEmpty()
    }

    private fun selfAndAncestors(folder: RelativePath): List<RelativePath> {
        if (folder.isRoot) {
            return emptyList()
        }
        return listOf(folder) + ancestorsOf(path = folder)
    }
}
