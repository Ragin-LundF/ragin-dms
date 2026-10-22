package dms.application.export

import dms.application.export.model.ExportOptionsModel
import dms.application.port.export.model.ExportBundleModel
import dms.application.port.export.model.ExportEntryModel
import dms.application.port.export.model.IndexRowModel
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldValues
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import java.time.ZoneId

/**
 * Lays out an export: the path of every file in the bundle and the index. Pure. Names stay unique
 * the way case-insensitive file systems compare them; a clash gets a ` (2)` suffix.
 */
object ExportPlanner {

    fun plan(
        catalog: CatalogModel,
        documents: List<DocumentModel>,
        options: ExportOptionsModel,
        zone: ZoneId,
    ): ExportBundleModel {
        val taken = mutableMapOf<String, MutableList<String>>()
        val placed = documents.sortedBy { it.path.value }.map { document ->
            val folder = if (options.flatten) RelativePathModel.ROOT else document.path.parent ?: RelativePathModel.ROOT
            val siblings = taken.getOrPut(key = folder.value.lowercase()) { mutableListOf() }
            val name = FileNamePolicy.nextFreeName(
                name = nameOf(document = document, options = options),
                siblings = siblings,
            )
            siblings += name
            document to folder.resolve(name = name).value
        }
        val definitions = catalog.settings.fieldDefinitions
        return ExportBundleModel(
            entries = placed.map { (document, target) -> ExportEntryModel(source = document.path, target = target) },
            index = if (options.index) {
                placed.map { (document, target) ->
                    rowOf(document = document, target = target, catalog = catalog, zone = zone)
                }
            } else {
                null
            },
            fieldLabels = definitions.map { it.label },
        )
    }

    private fun nameOf(document: DocumentModel, options: ExportOptionsModel): String {
        val title = document.metadata.title
        if (!options.renameToTitle || title == null) {
            return document.path.fileName
        }
        val extension = document.path.fileName.substringAfterLast(delimiter = '.', missingDelimiterValue = "")
        val base = FileNamePolicy.sanitize(name = title).take(n = FileNamePolicy.MAX_LENGTH - extension.length - 1)
        return if (extension.isEmpty()) base else "$base.$extension"
    }

    private fun rowOf(document: DocumentModel, target: String, catalog: CatalogModel, zone: ZoneId): IndexRowModel {
        val category = document.categoryId?.let { catalog.categories[it] }
        return IndexRowModel(
            path = target,
            title = document.metadata.title.orEmpty(),
            date = (document.metadata.documentDate ?: document.addedAt.atZone(zone).toLocalDate()).toString(),
            correspondent = document.metadata.correspondent.orEmpty(),
            category = category?.let { categoryPath(id = it.id, catalog = catalog) }.orEmpty(),
            tags = document.tags.sorted().joinToString(separator = ", ") { it.value },
            fields = catalog.settings.fieldDefinitions.map { definition ->
                document.fields[definition.key]?.let { FieldValues.format(value = it) }.orEmpty()
            },
            note = document.note.orEmpty(),
        )
    }

    private fun categoryPath(id: CategoryIdModel, catalog: CatalogModel): String {
        val categories = catalog.categories
        return generateSequence(seed = categories[id]) { current -> current.parentId?.let { categories[it] } }
            .take(n = categories.size)
            .map { it.name }
            .toList()
            .reversed()
            .joinToString(separator = " / ")
    }
}
