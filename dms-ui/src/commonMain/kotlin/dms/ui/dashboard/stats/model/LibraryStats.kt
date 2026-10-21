package dms.ui.dashboard.stats.model

import dms.ui.components.chart.model.ChartPoint

/**
 * The figures of the dashboard, derived from the catalog. [categories] and [tags] carry the
 * category id and the tag as [ChartPoint.id], so a click can open them in the library.
 */
data class LibraryStats(
    val documentCount: Int = 0,
    val totalBytes: Long = 0,
    val addedRecently: Int = 0,
    val uncategorizedCount: Int = 0,
    val byKind: List<KindCount> = emptyList(),
    val byYear: List<ChartPoint> = emptyList(),
    val categories: List<ChartPoint> = emptyList(),
    val tags: List<ChartPoint> = emptyList(),
    val correspondents: List<ChartPoint> = emptyList(),
    val recent: List<RecentDocument> = emptyList(),
) {
    /** The share of documents with a category, from 0 to 1. */
    val categorizedShare: Float
        get() {
            return if (documentCount == 0) 0f else (documentCount - uncategorizedCount).toFloat() / documentCount
        }
}
