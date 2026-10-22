package dms.domain.restructure

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.restructure.model.FlatFolderModel
import dms.domain.settings.model.InboxSettingsModel

/**
 * Finds folders that would profit from a restructure: folders without subfolders that hold more than
 * [DEFAULT_THRESHOLD] documents, or at least [MIN_SPLIT] documents that split cleanly into several
 * categories or years. The inbox is never one of them.
 */
object FolderShape {

    const val DEFAULT_THRESHOLD: Int = 40
    private const val MIN_SPLIT = 10
    private const val MIN_GROUPS = 2

    fun analyse(
        catalog: CatalogModel,
        inbox: InboxSettingsModel,
        threshold: Int = DEFAULT_THRESHOLD,
    ): List<FlatFolderModel> {
        val documents = catalog.documents.values.filterNot { inbox.contains(candidate = it.path) }
        val byFolder = documents.groupBy { it.path.parent ?: RelativePathModel.ROOT }
        val withSubfolders = documents.flatMap { document ->
            generateSequence(seed = document.path.parent?.parent) { it.parent }.toList()
        }.toSet()
        return byFolder
            .filterKeys { it !in withSubfolders && !(inbox.enabled && it == inbox.path) }
            .filter { (_, inside) -> inside.size > threshold || splitsCleanly(documents = inside) }
            .map { (folder, inside) -> FlatFolderModel(path = folder, fileCount = inside.size) }
            .sortedBy { it.path.value }
    }

    private fun splitsCleanly(documents: List<DocumentModel>): Boolean {
        if (documents.size < MIN_SPLIT) {
            return false
        }
        val categories = documents.map { it.categoryId }
        val years = documents.map { it.metadata.documentDate?.year }
        return (categories.none { it == null } && categories.distinct().size >= MIN_GROUPS) ||
            (years.none { it == null } && years.distinct().size >= MIN_GROUPS)
    }
}
