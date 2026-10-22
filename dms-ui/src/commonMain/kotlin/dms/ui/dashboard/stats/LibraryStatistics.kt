package dms.ui.dashboard.stats

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentModel
import dms.ui.components.chart.model.ChartPointModel
import dms.ui.dashboard.stats.model.KindCountModel
import dms.ui.dashboard.stats.model.LibraryStatsModel
import dms.ui.dashboard.stats.model.RecentDocumentModel
import dms.ui.library.LibraryFilter
import java.time.Duration
import java.time.Instant
import java.time.ZoneId
import java.util.Locale

/** Computes the dashboard's figures from the catalog; pure, so it is cheap to test. */
object LibraryStatistics {

    private const val YEARS = 8
    private const val TOP_CATEGORIES = 8
    private const val TOP_TAGS = 24
    private const val TOP_CORRESPONDENTS = 6
    private const val RECENT = 6
    private val RECENTLY: Duration = Duration.ofDays(30)

    fun of(catalog: CatalogModel?, now: Instant, zone: ZoneId): LibraryStatsModel {
        if (catalog == null) {
            return LibraryStatsModel()
        }
        val documents = catalog.documents.values
        return LibraryStatsModel(
            documentCount = documents.size,
            totalBytes = documents.sumOf { it.size },
            addedRecently = documents.count { it.addedAt > now.minus(RECENTLY) },
            uncategorizedCount = documents.count { it.categoryId == null },
            byKind = DocumentKind.entries
                .map { kind -> KindCountModel(kind = kind, count = documents.count { it.kind == kind }) }
                .filter { it.count > 0 },
            byYear = byYear(documents = documents, thisYear = now.atZone(zone).year, zone = zone),
            categories = documents
                .mapNotNull { document -> document.categoryId?.let { catalog.categories[it] } }
                .groupingBy { it }
                .eachCount()
                .map { (category, count) ->
                    ChartPointModel(id = category.id.value, label = category.name, value = count)
                }
                .ranked(limit = TOP_CATEGORIES),
            tags = documents
                .flatMap { it.tags }
                .groupingBy { it }
                .eachCount()
                .map { (tag, count) -> ChartPointModel(id = tag.value, label = tag.value, value = count) }
                .ranked(limit = TOP_TAGS),
            correspondents = documents
                .mapNotNull { it.metadata.correspondent }
                .groupingBy { it }
                .eachCount()
                .map { (name, count) -> ChartPointModel(id = name, label = name, value = count) }
                .ranked(limit = TOP_CORRESPONDENTS),
            recent = documents
                .sortedByDescending { it.addedAt }
                .take(n = RECENT)
                .map { recentOf(document = it, catalog = catalog) },
        )
    }

    /**
     * Documents per year of their document date, or of when they were added if they have none.
     * Shows up to [YEARS] years up to this one; older documents count towards the first bar.
     */
    private fun byYear(documents: Collection<DocumentModel>, thisYear: Int, zone: ZoneId): List<ChartPointModel> {
        if (documents.isEmpty()) {
            return emptyList()
        }
        val perYear = documents.groupingBy { LibraryFilter.yearOf(document = it, zone = zone) }
            .eachCount()
        val oldest = perYear.keys.min().coerceAtMost(maximumValue = thisYear)
        val first = maxOf(oldest, thisYear - YEARS + 1)
        return (first..thisYear).map { year ->
            val folded = year == first && oldest < first
            val count = if (folded) perYear.filterKeys { it <= year }.values.sum() else perYear[year] ?: 0
            val label = if (folded) "≤$year" else year.toString()
            ChartPointModel(id = label, label = label, value = count)
        }
    }

    private fun recentOf(document: DocumentModel, catalog: CatalogModel): RecentDocumentModel {
        return RecentDocumentModel(
            id = document.id,
            title = document.metadata.title ?: document.path.fileName,
            folder = document.path.parent?.value.orEmpty(),
            kind = document.kind,
            categoryName = document.categoryId?.let { catalog.categories[it]?.name },
            addedAt = document.addedAt,
        )
    }

    /** Largest first, ties by name, at most [limit]. */
    private fun List<ChartPointModel>.ranked(limit: Int): List<ChartPointModel> {
        val order = compareByDescending<ChartPointModel> { it.value }.thenBy { it.label.lowercase(Locale.ROOT) }
        return sortedWith(comparator = order)
            .take(n = limit)
    }
}
