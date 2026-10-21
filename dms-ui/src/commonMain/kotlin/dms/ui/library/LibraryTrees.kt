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

    /** The root first, then every folder in path order, each followed by its subfolders. */
    fun folders(documents: Collection<Document>, createdFolders: Set<RelativePath>): List<FolderEntry> {
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
        )
        return listOf(root) + folders.map { folder ->
            FolderEntry(
                path = folder,
                name = folder.fileName,
                depth = folder.segments.size,
                documentCount = documents.count { it.path.isInside(folder = folder) },
            )
        }
    }

    /** Top-level categories by name, each followed by its subcategories. */
    fun categories(categories: Collection<Category>, documents: Collection<Document>): List<CategoryEntry> {
        val children = categories.groupBy { it.parentId }
        val counts = documents.groupingBy { it.categoryId }.eachCount()
        return flatten(parentId = null, depth = 0, children = children, counts = counts)
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
