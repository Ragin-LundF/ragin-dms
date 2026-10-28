package dms.ui.shell.palette

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.ui.library.model.LibrarySelection
import dms.ui.review.ReviewDrafts
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.OperationLane
import dms.ui.shell.palette.model.PaletteAction
import dms.ui.shell.palette.model.PaletteGroupModel
import dms.ui.shell.palette.model.PaletteItemModel
import java.util.Locale

/**
 * What the palette lists for a query: documents by name and by text, places to go (tags, categories,
 * folders, smart folders) with their counts, and actions, which say why they wait while their lane is
 * busy. Only reads the catalog.
 */
object PaletteItems {

    private const val MAX_DOCUMENTS = 6
    private const val MAX_TEXT_MATCHES = 4
    private const val MAX_PLACES = 8

    /**
     * Every place of [catalog] with its count, counted in one pass over the documents. Computed once per
     * catalog, so typing only filters.
     */
    fun placesOf(catalog: CatalogModel?): List<PaletteItemModel.GoTo> {
        if (catalog == null) {
            return emptyList()
        }
        val documents = catalog.documents.values
        val tagCounts = documents.flatMap { it.tags }.groupingBy { it }.eachCount()
        val categoryCounts = documents.mapNotNull { it.categoryId }.groupingBy { it }.eachCount()
        val folderCounts = documents.flatMap { foldersOf(path = it.path) }.groupingBy { it }.eachCount()
        val tags = catalog.tags.sorted().map { tag ->
            PaletteItemModel.GoTo(
                selection = LibrarySelection.Tag(tag = tag),
                label = tag.value,
                kind = "Tag",
                count = tagCounts[tag] ?: 0,
            )
        }
        val categories = catalog.categories.values.map { category ->
            PaletteItemModel.GoTo(
                selection = LibrarySelection.Category(categoryId = category.id),
                label = ReviewDrafts.pathOf(category = category, catalog = catalog),
                kind = "Category",
                count = categoryCounts[category.id] ?: 0,
            )
        }.sortedBy { it.label.lowercase(Locale.ROOT) }
        val folders = folderCounts.entries.sortedBy { it.key.value.lowercase(Locale.ROOT) }.map { (folder, count) ->
            PaletteItemModel.GoTo(
                selection = LibrarySelection.Folder(path = folder),
                label = folder.value,
                kind = "Folder",
                count = count,
            )
        }
        val smartFolders = catalog.settings.savedSearches.map { search ->
            PaletteItemModel.GoTo(
                selection = LibrarySelection.SmartFolder(id = search.id),
                label = search.name,
                kind = "Smart folder",
                count = null,
            )
        }
        return tags + categories + folders + smartFolders
    }

    /**
     * The groups for [query]; [busy] names what runs in each lane, if anything. Without [aiEnabled], no
     * AI action and no question to the library is offered.
     */
    fun of(
        query: String,
        catalog: CatalogModel?,
        textMatches: Set<DocumentIdModel>,
        places: List<PaletteItemModel.GoTo>,
        busy: Map<OperationLane, String>,
        aiEnabled: Boolean = true,
    ): List<PaletteGroupModel> {
        val wanted = query.trim().lowercase(Locale.ROOT)
        val documents = catalog?.let { documents(catalog = it, wanted = wanted, textMatches = textMatches) }.orEmpty()
        val matchingPlaces = if (wanted.isEmpty()) {
            emptyList()
        } else {
            places.asSequence().filter { it.label.lowercase(Locale.ROOT).contains(other = wanted) }
                .take(n = MAX_PLACES)
                .toList()
        }
        return listOf(
            PaletteGroupModel(title = "Documents", items = documents),
            PaletteGroupModel(title = "Go to", items = matchingPlaces),
            PaletteGroupModel(
                title = "Actions",
                items = actions(query = query.trim(), wanted = wanted, busy = busy, aiEnabled = aiEnabled),
            ),
        ).filter { it.items.isNotEmpty() }
    }

    private fun documents(
        catalog: CatalogModel,
        wanted: String,
        textMatches: Set<DocumentIdModel>,
    ): List<PaletteItemModel> {
        if (wanted.isEmpty()) {
            return emptyList()
        }
        val byName = catalog.documents.values.asSequence()
            .filter { document -> namesOf(document = document).any { it.contains(other = wanted) } }
            .sortedBy { it.path.value.lowercase(Locale.ROOT) }
            .take(n = MAX_DOCUMENTS)
            .toList()
        val byText = textMatches.mapNotNull { catalog.documents[it] }
            .filterNot { it in byName }
            .sortedBy { it.path.value.lowercase(Locale.ROOT) }
            .take(n = MAX_TEXT_MATCHES)
        return byName.map { itemOf(document = it, detail = it.path.value) } +
            byText.map { itemOf(document = it, detail = "Found in its text · ${it.path.value}") }
    }

    private fun actions(
        query: String,
        wanted: String,
        busy: Map<OperationLane, String>,
        aiEnabled: Boolean,
    ): List<PaletteItemModel> {
        val matching = PaletteAction.entries
            .filter { aiEnabled || it.lane != OperationLane.AI }
            .filter { wanted.isEmpty() || it.label.lowercase(Locale.ROOT).contains(other = wanted) }
            .map { action ->
                val running = action.lane?.let { busy[it] }
                PaletteItemModel.Run(
                    action = action,
                    disabledReason = running?.let { OutcomeMessages.waitFor(operation = it) },
                )
            }
        val ask = PaletteItemModel.AskLibrary(question = query).takeIf { aiEnabled && query.isNotEmpty() }
        return matching + listOfNotNull(ask)
    }

    private fun namesOf(document: DocumentModel): List<String> {
        return listOfNotNull(document.path.fileName, document.metadata.title, document.metadata.correspondent)
            .map { it.lowercase(Locale.ROOT) }
    }

    private fun itemOf(document: DocumentModel, detail: String): PaletteItemModel {
        return PaletteItemModel.OpenDocument(
            documentId = document.id,
            label = document.metadata.title ?: document.path.fileName,
            detail = detail,
        )
    }

    /** Every folder above a file. */
    private fun foldersOf(path: RelativePathModel): List<RelativePathModel> {
        return generateSequence(seed = path.parent) { it.parent }.filterNot { it.isRoot }.toList()
    }
}
