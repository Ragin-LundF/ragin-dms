package dms.ui.dashboard.stats

import dms.ui.components.chart.model.ChartPointModel
import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.dashboard.model.FolderHealth
import dms.ui.dashboard.model.FolderStatusModel
import dms.ui.dashboard.organize.model.InboxSummaryModel
import dms.ui.dashboard.stats.model.LibraryStatsModel
import dms.ui.dashboard.stats.model.StatColumnModel
import dms.ui.dashboard.stats.model.StatLink
import kotlin.test.Test
import kotlin.test.assertEquals

class StatColumnsTest {

    private val years = listOf(
        ChartPointModel(id = "2025", label = "2025", value = 3),
        ChartPointModel(id = "2026", label = "2026", value = 5),
    )

    @Test
    fun `the strip shows documents, categorized share, inbox and storage in this order`() {
        val state = DashboardUiStateModel(
            folders = listOf(folder(name = "Primary"), folder(name = "Backup")),
            stats = LibraryStatsModel(
                documentCount = 8,
                totalBytes = 2048,
                addedRecently = 2,
                growth = listOf(6, 7, 8),
                uncategorizedCount = 2,
                byYear = years,
            ),
            inbox = InboxSummaryModel(path = "Inbox", count = 4, aiAnalysis = false),
        )

        val columns = StatColumns.of(state = state)

        assertEquals(
            expected = listOf(
                StatColumnModel(
                    label = "Documents",
                    value = "8",
                    caption = "+2 in the last 30 days",
                    trend = listOf(6, 7, 8),
                ),
                StatColumnModel(
                    label = "Categorized",
                    value = "75 %",
                    caption = "2 without a category",
                    meter = 0.75f,
                    link = StatLink.UNCATEGORIZED,
                ),
                StatColumnModel(label = "Inbox", value = "4", caption = "Waiting to be filed", link = StatLink.INBOX),
                StatColumnModel(label = "Storage", value = "2.0 KB", caption = "Kept in each of 2 library folders"),
            ),
            actual = columns,
        )
    }

    @Test
    fun `without additions this month the sparkline shows the years instead of a flat line`() {
        val state = DashboardUiStateModel(
            stats = LibraryStatsModel(
                documentCount = 8,
                addedRecently = 0,
                growth = List(size = 30) { 8 },
                byYear = years,
            ),
        )

        val documents = StatColumns.of(state = state).first()

        assertEquals(expected = listOf(3, 5), actual = documents.trend)
        assertEquals(expected = "None added lately", actual = documents.caption)
    }

    @Test
    fun `a fully categorized library and a switched off inbox link nowhere`() {
        val state = DashboardUiStateModel(stats = LibraryStatsModel(documentCount = 3, uncategorizedCount = 0))

        val columns = StatColumns.of(state = state)

        assertEquals(expected = listOf(null, null, null, null), actual = columns.map { it.link })
        assertEquals(expected = "100 %", actual = columns[1].value)
        assertEquals(
            expected = "Off" to "Switched off in the settings",
            actual = columns[2].let { it.value to it.caption },
        )
    }

    private fun folder(name: String): FolderStatusModel {
        return FolderStatusModel(
            name = name,
            location = "/$name",
            isPrimary = name == "Primary",
            health = FolderHealth.REACHABLE,
            pendingOperations = 0,
        )
    }
}
