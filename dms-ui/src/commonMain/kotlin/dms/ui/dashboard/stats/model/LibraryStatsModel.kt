package dms.ui.dashboard.stats.model

import dms.ui.components.chart.model.ChartPointModel

/**
 * The figures of the dashboard, derived from the catalog. [categories] and [tags] carry the
 * category id and the tag as [ChartPointModel.id], so a click can open them in the library.
 * [growth] holds the size of the library at the end of each of the last 30 days, oldest first.
 */
data class LibraryStatsModel(
    val documentCount: Int = 0,
    val totalBytes: Long = 0,
    val addedRecently: Int = 0,
    val growth: List<Int> = emptyList(),
    val uncategorizedCount: Int = 0,
    val byKind: List<KindCountModel> = emptyList(),
    val byYear: List<ChartPointModel> = emptyList(),
    val categories: List<ChartPointModel> = emptyList(),
    val tags: List<ChartPointModel> = emptyList(),
    val correspondents: List<ChartPointModel> = emptyList(),
    val recent: List<RecentDocumentModel> = emptyList(),
) {
    /** The share of documents with a category, from 0 to 1. */
    val categorizedShare: Float
        get() {
            return if (documentCount == 0) 0f else (documentCount - uncategorizedCount).toFloat() / documentCount
        }
}
