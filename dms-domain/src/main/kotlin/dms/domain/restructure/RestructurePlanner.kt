package dms.domain.restructure

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.RelativePathModel
import dms.domain.restructure.model.PlannedMoveModel
import dms.domain.restructure.model.RestructurePlanModel
import dms.domain.restructure.model.SkippedDocumentModel
import dms.domain.restructure.template.PlaceholderName
import dms.domain.restructure.template.TemplateRendering
import dms.domain.restructure.template.model.PathTemplateModel
import dms.domain.settings.model.InboxSettingsModel

/**
 * Plans a restructure: where each document of a scope belongs by a [PathTemplateModel]. Pure; nothing
 * moves until the user applied the plan.
 *
 * A target is never a path another document has now, even one that moves away in the same plan, so
 * the moves can be applied one by one in any order. A clash gets a ` (2)` suffix; names that differ
 * only by case or Unicode normalization clash, as they do on macOS and Windows. A new folder that
 * differs from an existing one only by case takes the existing folder's spelling.
 */
object RestructurePlanner {

    fun plan(
        catalog: CatalogModel,
        scope: List<DocumentModel>,
        template: PathTemplateModel,
        naming: FileNaming,
        inbox: InboxSettingsModel,
    ): RestructurePlanModel {
        val occupied = catalog.documents.values.groupBy { key(path = it.path) }.mapValues { (_, documents) ->
            documents.map { it.id }.toSet()
        }.toMutableMap()
        val folders = existingFolders(catalog = catalog)
        val moves = mutableListOf<PlannedMoveModel>()
        val unchanged = mutableListOf<DocumentIdModel>()
        val skipped = mutableListOf<SkippedDocumentModel>()
        val renamed = mutableSetOf<DocumentIdModel>()
        scope.sortedBy { it.path.value }.forEach { document ->
            if (inbox.contains(candidate = document.path)) {
                skipped += SkippedDocumentModel(
                    documentId = document.id,
                    path = document.path,
                    reason = SkipReason.IN_INBOX,
                )
                return@forEach
            }
            when (val rendering = template.render(document = document, catalog = catalog)) {
                is TemplateRendering.Missing -> skipped += SkippedDocumentModel(
                    documentId = document.id,
                    path = document.path,
                    reason = reasonFor(placeholder = rendering.placeholder),
                )

                is TemplateRendering.Rendered -> {
                    val folder = spelledLikeExisting(folder = rendering.folder, existing = folders)
                    val wanted = folder.resolve(name = fileNameFor(document = document, naming = naming))
                    if (key(path = wanted) == key(path = document.path)) {
                        unchanged += document.id
                        return@forEach
                    }
                    val target = freeTarget(wanted = wanted, occupied = occupied)
                    if (target != wanted) {
                        renamed += document.id
                    }
                    occupied[key(path = target)] = setOf(document.id)
                    moves += PlannedMoveModel(documentId = document.id, from = document.path, to = target)
                }
            }
        }
        return RestructurePlanModel(
            template = template.text,
            moves = moves,
            unchanged = unchanged,
            skipped = skipped,
            renamedForClash = renamed,
        )
    }

    private fun freeTarget(wanted: RelativePathModel, occupied: Map<String, Set<DocumentIdModel>>): RelativePathModel {
        if (key(path = wanted) !in occupied) {
            return wanted
        }
        val parent = wanted.parent ?: RelativePathModel.ROOT
        val siblings = occupied.keys
            .filter { it.substringBeforeLast(delimiter = '/', missingDelimiterValue = "") == key(path = parent) }
            .map { it.substringAfterLast(delimiter = '/') }
        return parent.resolve(name = FileNamePolicy.nextFreeName(name = wanted.fileName, siblings = siblings))
    }

    private fun fileNameFor(document: DocumentModel, naming: FileNaming): String {
        val title = document.metadata.title
        val date = document.metadata.documentDate
        if (naming == FileNaming.KEEP || title == null || date == null) {
            return document.path.fileName
        }
        val extension = document.path.fileName.substringAfterLast(delimiter = '.', missingDelimiterValue = "")
        val base = FileNamePolicy.sanitize(
            name = "$date $title",
        ).take(n = FileNamePolicy.MAX_LENGTH - extension.length - 1)
        return if (extension.isEmpty()) base else "$base.$extension"
    }

    /** Every folder the catalog's documents are in, by their comparison key. */
    private fun existingFolders(catalog: CatalogModel): Map<String, RelativePathModel> {
        return catalog.documents.values
            .flatMap { document -> ancestorsOf(path = document.path) }
            .associateBy { key(path = it) }
    }

    private fun ancestorsOf(path: RelativePathModel): List<RelativePathModel> {
        return generateSequence(seed = path.parent) { it.parent }.filterNot { it.isRoot }.toList()
    }

    /** [folder] with each level spelled like an existing folder that differs only by case. */
    private fun spelledLikeExisting(
        folder: RelativePathModel,
        existing: Map<String,
        RelativePathModel>,
    ): RelativePathModel {
        var result = RelativePathModel.ROOT
        folder.segments.forEach { segment ->
            val candidate = result.resolve(name = segment)
            result = existing[key(path = candidate)] ?: candidate
        }
        return result
    }

    private fun reasonFor(placeholder: PlaceholderName): SkipReason {
        return when (placeholder) {
            PlaceholderName.CATEGORY, PlaceholderName.CATEGORY_TOP -> SkipReason.MISSING_CATEGORY
            PlaceholderName.YEAR, PlaceholderName.MONTH -> SkipReason.MISSING_DATE
            PlaceholderName.CORRESPONDENT -> SkipReason.MISSING_CORRESPONDENT
            PlaceholderName.FIELD, PlaceholderName.KIND -> SkipReason.MISSING_FIELD
        }
    }

    private fun key(path: RelativePathModel): String {
        return FileNamePolicy.comparisonKey(name = path.value)
    }
}
