package dms.ui.dashboard.stats

import dms.domain.catalog.Catalog
import dms.domain.document.Document
import dms.domain.document.DocumentKind
import dms.ui.components.chart.model.ChartPoint
import dms.ui.dashboard.stats.model.KindCount
import dms.ui.dashboard.stats.model.LibraryStats
import dms.ui.dashboard.stats.model.RecentDocument
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

    fun of(catalog: Catalog?, now: Instant, zone: ZoneId): LibraryStats {
        if (catalog == null) {
            return LibraryStats()
        }
        val documents = catalog.documents.values
        return LibraryStats(
            documentCount = documents.size,
            totalBytes = documents.sumOf { it.size },
            addedRecently = documents.count { it.addedAt > now.minus(RECENTLY) },
            uncategorizedCount = documents.count { it.categoryId == null },
            byKind = DocumentKind.entries
                .map { kind -> KindCount(kind = kind, count = documents.count { it.kind == kind }) }
                .filter { it.count > 0 },
            byYear = byYear(documents = documents, thisYear = now.atZone(zone).year, zone = zone),
            categories = documents
                .mapNotNull { document -> document.categoryId?.let { catalog.categories[it] } }
                .groupingBy { it }
                .eachCount()
                .map { (category, count) -> ChartPoint(id = category.id.value, label = category.name, value = count) }
                .ranked(limit = TOP_CATEGORIES),
            tags = documents
                .flatMap { it.tags }
                .groupingBy { it }
                .eachCount()
                .map { (tag, count) -> ChartPoint(id = tag.value, label = tag.value, value = count) }
                .ranked(limit = TOP_TAGS),
            correspondents = documents
                .mapNotNull { it.metadata.correspondent }
                .groupingBy { it }
                .eachCount()
                .map { (name, count) -> ChartPoint(id = name, label = name, value = count) }
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
    private fun byYear(documents: Collection<Document>, thisYear: Int, zone: ZoneId): List<ChartPoint> {
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
            ChartPoint(id = label, label = label, value = count)
        }
    }

    private fun recentOf(document: Document, catalog: Catalog): RecentDocument {
        return RecentDocument(
            id = document.id,
            title = document.metadata.title ?: document.path.fileName,
            folder = document.path.parent?.value.orEmpty(),
            kind = document.kind,
            categoryName = document.categoryId?.let { catalog.categories[it]?.name },
            addedAt = document.addedAt,
        )
    }

    /** Largest first, ties by name, at most [limit]. */
    private fun List<ChartPoint>.ranked(limit: Int): List<ChartPoint> {
        val order = compareByDescending<ChartPoint> { it.value }.thenBy { it.label.lowercase(Locale.ROOT) }
        return sortedWith(comparator = order)
            .take(n = limit)
    }
}
