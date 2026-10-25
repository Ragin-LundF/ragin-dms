package dms.ui.library.filter

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.query.model.DateRangeModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.ui.components.surface.KindStyles
import dms.ui.library.filter.model.Facet
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.filter.model.FilterTokenModel
import dms.ui.library.model.YearFilterModel
import java.util.Locale

/**
 * The facet filters as tokens of the search field, a saved view's query as tokens that stay, and the
 * tokens the typed text suggests. Tokens and facet chips render the same filters, so both always agree.
 */
object FilterTokens {

    private const val MIN_TAG_LETTERS = 2
    private const val MIN_CORRESPONDENT_LETTERS = 3
    private const val MAX_SUGGESTIONS = 3
    private const val CATEGORY_SEPARATOR = " › "
    private const val DECEMBER = 12
    private const val LAST_DAY = 31
    private val FOUR_DIGIT_YEAR = Regex(pattern = "^\\d{4}$")

    fun facetOf(filter: FacetFilter): Facet {
        return when (filter) {
            is FacetFilter.Category -> Facet.CATEGORY
            is FacetFilter.Tag -> Facet.TAG
            is FacetFilter.Year -> Facet.YEAR
            is FacetFilter.Correspondent -> Facet.CORRESPONDENT
            is FacetFilter.Type -> Facet.TYPE
            is FacetFilter.Folder -> Facet.FOLDER
        }
    }

    /** [filters] with [filter] at the end, in place of the filter its facet had. */
    fun with(filters: List<FacetFilter>, filter: FacetFilter): List<FacetFilter> {
        val facet = facetOf(filter = filter)
        return filters.filterNot { facetOf(filter = it) == facet } + filter
    }

    fun tokenOf(filter: FacetFilter, catalog: CatalogModel): FilterTokenModel {
        return FilterTokenModel(
            key = facetOf(filter = filter).key,
            value = valueOf(filter = filter, catalog = catalog),
            filter = filter,
        )
    }

    /** What a facet chip shows while [filter] is active, such as `Home › Energy` or `PDF`. */
    fun valueOf(filter: FacetFilter, catalog: CatalogModel): String {
        return when (filter) {
            is FacetFilter.Category -> filter.categoryId?.let { categoryPath(categoryId = it, catalog = catalog) }
                ?: "Without a category"

            is FacetFilter.Tag -> filter.tag.value
            is FacetFilter.Year -> filter.year.label
            is FacetFilter.Correspondent -> filter.name
            is FacetFilter.Type -> KindStyles.label(kind = filter.kind)
            is FacetFilter.Folder -> filter.path.value
        }
    }

    /** The category with its parents, as in `Home › Energy`. */
    fun categoryPath(categoryId: CategoryIdModel, catalog: CatalogModel): String {
        val names = mutableListOf<String>()
        val visited = mutableSetOf<CategoryIdModel>()
        var current = catalog.categories[categoryId]
        while (current != null && visited.add(element = current.id)) {
            names += current.name
            current = current.parentId?.let { catalog.categories[it] }
        }
        return names.reversed().joinToString(separator = CATEGORY_SEPARATOR).ifEmpty { "Unknown category" }
    }

    /** The conditions a saved view keeps in its query, as tokens without a way to remove them. */
    fun savedTokens(query: DocumentQueryModel, catalog: CatalogModel): List<FilterTokenModel> {
        val tags = query.tags
        return listOfNotNull(
            query.text?.takeIf { it.isNotBlank() }?.let { FilterTokenModel(key = "text", value = it) },
            query.contentText?.takeIf { it.isNotBlank() }?.let { FilterTokenModel(key = "text", value = it) },
            query.category?.let { id ->
                FilterTokenModel(key = "category", value = categoryPath(categoryId = id, catalog = catalog))
            },
            FilterTokenModel(key = "category", value = "Without a category").takeIf { query.uncategorized },
            FilterTokenModel(key = "tag", value = "none").takeIf { query.untagged },
            tags.any.takeIf { it.isNotEmpty() }?.let { any ->
                FilterTokenModel(key = "tag", value = "any of ${any.sorted().joinToString { it.value }}")
            },
            query.dates?.let { FilterTokenModel(key = dateKeyOf(range = it), value = dateValueOf(range = it)) },
            query.correspondent?.let { FilterTokenModel(key = "from", value = it) },
            query.folder?.takeUnless { it.isRoot }?.let { FilterTokenModel(key = "folder", value = it.value) },
            query.addedSince?.let { FilterTokenModel(key = "added", value = "since $it") },
        ) + tags.all.sorted().map { FilterTokenModel(key = "tag", value = it.value) } +
            tags.none.sorted().map { FilterTokenModel(key = "not tag", value = it.value) } +
            query.fields.map { condition ->
                val label = catalog.settings.field(key = condition.key)?.label ?: condition.key.value
                FilterTokenModel(key = "field", value = label)
            }
    }

    /**
     * The tokens [typed] text suggests: a known tag, a four-digit year or a correspondent of [documents],
     * at most three, without the filters already in [active].
     */
    fun suggestions(
        typed: String,
        documents: Collection<DocumentModel>,
        active: List<FacetFilter>,
        catalog: CatalogModel,
    ): List<FilterTokenModel> {
        val wanted = typed.trim().lowercase(Locale.ROOT)
        if (wanted.isEmpty()) {
            return emptyList()
        }
        val tags = catalog.tags.sorted()
            .filter { wanted.length >= MIN_TAG_LETTERS }
            .filter { it.value.lowercase(Locale.ROOT).contains(other = wanted) }
            .map { FacetFilter.Tag(tag = it) }
        val year = wanted.takeIf { FOUR_DIGIT_YEAR.matches(input = it) }
            ?.let { FacetFilter.Year(year = YearFilterModel(year = it.toInt())) }
        val correspondents = documents.mapNotNull { it.metadata.correspondent?.trim() }
            .distinctBy { it.lowercase(Locale.ROOT) }
            .sortedBy { it.lowercase(Locale.ROOT) }
            .filter { wanted.length >= MIN_CORRESPONDENT_LETTERS }
            .filter { it.lowercase(Locale.ROOT).contains(other = wanted) }
            .map { FacetFilter.Correspondent(name = it) }
        return (tags + listOfNotNull(year) + correspondents)
            .filterNot { it in active }
            .take(n = MAX_SUGGESTIONS)
            .map { tokenOf(filter = it, catalog = catalog) }
    }

    private fun dateKeyOf(range: DateRangeModel): String {
        return if (yearOf(range = range) != null) "year" else "date"
    }

    private fun dateValueOf(range: DateRangeModel): String {
        val year = yearOf(range = range)
        if (year != null) {
            return year.label
        }
        return "${range.from ?: "…"} – ${range.to ?: "…"}"
    }

    /** The year filter [range] stands for, if it covers exactly one year or ends with one. */
    private fun yearOf(range: DateRangeModel): YearFilterModel? {
        val to = range.to ?: return null
        val endsWithYear = to.monthValue == DECEMBER && to.dayOfMonth == LAST_DAY
        return when {
            !endsWithYear -> null
            range.from == null -> YearFilterModel(year = to.year, andEarlier = true)
            range == DateRangeModel.year(year = to.year) -> YearFilterModel(year = to.year)
            else -> null
        }
    }
}
