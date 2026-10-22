package dms.domain.outline.model

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel

/**
 * The library's folder tree as the AI sees it when it suggests where to file a document. Folders the
 * user excluded (the inbox) and hidden folders are left out; a large tree is cut at the depth that
 * keeps it within the limit, and the cut folders count what they hide.
 */
data class FolderOutlineModel(
    val folders: List<OutlineFolderModel>,
    val excluded: List<RelativePathModel> = emptyList(),
) {

    /** `true` if [path] is a folder of the outline. */
    fun contains(path: RelativePathModel): Boolean {
        return folders.any { it.path == path }
    }

    /**
     * `true` if [path] may be created as a new folder: directly below a folder of the outline or below
     * the root, and not one of the excluded folders or inside one.
     */
    fun canCreate(path: RelativePathModel): Boolean {
        val parent = path.parent ?: return false
        if (excluded.any { path == it || path.isInside(folder = it) }) {
            return false
        }
        return parent.isRoot || contains(path = parent)
    }

    companion object {
        const val DEFAULT_LIMIT: Int = 200
        private const val SAMPLE_TITLES = 3

        fun of(
            catalog: CatalogModel,
            excluded: List<RelativePathModel>,
            limit: Int = DEFAULT_LIMIT,
        ): FolderOutlineModel {
            val documents = catalog.documents.values.filter { document ->
                excluded.none { document.path.isInside(folder = it) }
            }
            val all = documents.flatMap { ancestorsOf(path = it.path) }
                .distinct()
                .filter { folder -> excluded.none { folder == it || folder.isInside(folder = it) } }
                .filterNot { folder -> folder.segments.any { it.startsWith(prefix = ".") } }
            val depth = depthWithin(folders = all, limit = limit)
            val shown = all.filter { it.segments.size <= depth }.sortedBy { it.value }.take(n = limit)
            return FolderOutlineModel(
                excluded = excluded,
                folders = shown.map { folder ->
                    val inside = documents.filter { it.path.isInside(folder = folder) }
                    OutlineFolderModel(
                        path = folder,
                        documentCount = inside.size,
                        topCategory = inside.mapNotNull { it.categoryId }.groupingBy { it }.eachCount()
                            .maxWithOrNull(compareBy({ it.value }, { it.key.value }))?.key,
                        sampleTitles = inside.sortedByDescending { it.addedAt }
                            .take(n = SAMPLE_TITLES)
                            .map { titleOf(document = it) },
                        hiddenSubfolders = all.count { it.isInside(folder = folder) && it.segments.size > depth },
                    )
                },
            )
        }

        /** Every folder above [path], from the top level down, without the root. */
        private fun ancestorsOf(path: RelativePathModel): List<RelativePathModel> {
            return generateSequence(seed = path.parent) { it.parent }.filterNot { it.isRoot }.toList()
        }

        /** The deepest level at which the outline still has at most [limit] folders. */
        private fun depthWithin(folders: List<RelativePathModel>, limit: Int): Int {
            var depth = folders.maxOfOrNull { it.segments.size } ?: 0
            while (depth > 1 && folders.count { it.segments.size <= depth } > limit) {
                depth--
            }
            return depth
        }

        private fun titleOf(document: DocumentModel): String {
            return document.metadata.title ?: document.path.fileName
        }
    }
}
