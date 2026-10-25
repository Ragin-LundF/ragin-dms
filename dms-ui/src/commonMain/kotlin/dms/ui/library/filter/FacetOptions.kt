package dms.ui.library.filter

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.ui.components.surface.KindStyles
import dms.ui.library.LibraryFilter
import dms.ui.library.LibraryTrees
import dms.ui.library.filter.model.Facet
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.filter.model.FacetOptionModel
import dms.ui.library.model.YearFilterModel
import java.time.ZoneId
import java.util.Locale

/**
 * The values each facet chip offers, with how many of [documents] have them: categories as a tree with
 * "Without a category" at the end, tags and correspondents by how often they occur, years newest first,
 * the kinds of file, and the folders as a tree.
 */
object FacetOptions {

    private val byCount: Comparator<FacetOptionModel> =
        compareByDescending<FacetOptionModel> { it.count }.thenBy { it.label.lowercase(Locale.ROOT) }

    fun of(
        documents: Collection<DocumentModel>,
        catalog: CatalogModel,
        zone: ZoneId,
    ): Map<Facet, List<FacetOptionModel>> {
        return Facet.entries.associateWith { facet ->
            when (facet) {
                Facet.CATEGORY -> categories(documents = documents, catalog = catalog)
                Facet.TAG -> tags(documents = documents, catalog = catalog)
                Facet.YEAR -> years(documents = documents, zone = zone)
                Facet.CORRESPONDENT -> correspondents(documents = documents)
                Facet.TYPE -> kinds(documents = documents)
                Facet.FOLDER -> folders(documents = documents)
            }
        }
    }

    private fun categories(documents: Collection<DocumentModel>, catalog: CatalogModel): List<FacetOptionModel> {
        val counts = mutableMapOf<CategoryIdModel, Int>()
        documents.forEach { document ->
            selfAndParents(categoryId = document.categoryId, catalog = catalog).forEach { id ->
                counts[id] = (counts[id] ?: 0) + 1
            }
        }
        val tree = LibraryTrees.categories(categories = catalog.categories.values, documents = documents)
        val options = tree.map { entry ->
            FacetOptionModel(
                filter = FacetFilter.Category(categoryId = entry.category.id),
                label = entry.category.name,
                count = counts[entry.category.id] ?: 0,
                depth = entry.depth,
            )
        }
        val without = FacetOptionModel(
            filter = FacetFilter.Category(categoryId = null),
            label = "Without a category",
            count = documents.count { it.categoryId == null },
            missing = true,
        )
        return options + without
    }

    private fun tags(documents: Collection<DocumentModel>, catalog: CatalogModel): List<FacetOptionModel> {
        val counts = documents.flatMap { it.tags }.groupingBy { it }.eachCount()
        return catalog.tags
            .map { tag ->
                FacetOptionModel(filter = FacetFilter.Tag(tag = tag), label = tag.value, count = counts[tag] ?: 0)
            }
            .sortedWith(comparator = byCount)
    }

    private fun years(documents: Collection<DocumentModel>, zone: ZoneId): List<FacetOptionModel> {
        val counts = documents.groupingBy { LibraryFilter.yearOf(document = it, zone = zone) }.eachCount()
        return counts.keys.sortedDescending().map { year ->
            FacetOptionModel(
                filter = FacetFilter.Year(year = YearFilterModel(year = year)),
                label = year.toString(),
                count = counts.getValue(key = year),
            )
        }
    }

    private fun correspondents(documents: Collection<DocumentModel>): List<FacetOptionModel> {
        return documents.mapNotNull { it.metadata.correspondent?.trim() }
            .groupBy { lowercase(text = it) }
            .values
            .map { spellings ->
                val name = spellings.first()
                FacetOptionModel(filter = FacetFilter.Correspondent(name = name), label = name, count = spellings.size)
            }
            .sortedWith(comparator = byCount)
    }

    private fun kinds(documents: Collection<DocumentModel>): List<FacetOptionModel> {
        val counts = documents.groupingBy { it.kind }.eachCount()
        return DocumentKind.entries.filter { it in counts }.map { kind ->
            FacetOptionModel(
                filter = FacetFilter.Type(kind = kind),
                label = KindStyles.label(kind = kind),
                count = counts.getValue(key = kind),
            )
        }
    }

    private fun folders(documents: Collection<DocumentModel>): List<FacetOptionModel> {
        return LibraryTrees.folders(documents = documents, foldersOnDisk = emptySet())
            .filterNot { it.path.isRoot }
            .map { entry ->
                FacetOptionModel(
                    filter = FacetFilter.Folder(path = entry.path),
                    label = entry.name,
                    count = entry.documentCount,
                    depth = entry.depth - 1,
                )
            }
    }

    private fun selfAndParents(categoryId: CategoryIdModel?, catalog: CatalogModel): Set<CategoryIdModel> {
        val chain = mutableSetOf<CategoryIdModel>()
        var current = categoryId
        while (current != null && chain.add(element = current)) {
            current = catalog.categories[current]?.parentId
        }
        return chain
    }

    private fun lowercase(text: String): String {
        return text.lowercase(Locale.ROOT)
    }
}
