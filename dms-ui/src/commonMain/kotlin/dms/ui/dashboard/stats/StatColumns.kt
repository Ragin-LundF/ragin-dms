package dms.ui.dashboard.stats

import dms.ui.components.format.FileSizes
import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.dashboard.stats.model.StatColumnModel
import dms.ui.dashboard.stats.model.StatLink
import java.util.Locale
import kotlin.math.roundToInt

/** The four figures of the stat strip: documents, how much is categorized, the inbox and the storage. */
object StatColumns {

    private const val PERCENT = 100

    fun of(state: DashboardUiStateModel): List<StatColumnModel> {
        return listOf(
            documents(state = state),
            categorized(state = state),
            inbox(state = state),
            storage(state = state),
        )
    }

    private fun documents(state: DashboardUiStateModel): StatColumnModel {
        val stats = state.stats
        // A month without additions draws a flat line, so the years tell more then.
        val trend = if (stats.addedRecently > 0) stats.growth else stats.byYear.map { it.value }
        return StatColumnModel(
            label = "Documents",
            value = "%,d".format(locale = Locale.getDefault(), stats.documentCount),
            caption = if (stats.addedRecently > 0) {
                "+${stats.addedRecently} in the last 30 days"
            } else {
                "None added lately"
            },
            trend = trend,
        )
    }

    private fun categorized(state: DashboardUiStateModel): StatColumnModel {
        val stats = state.stats
        val missing = stats.uncategorizedCount
        return StatColumnModel(
            label = "Categorized",
            value = "${(stats.categorizedShare * PERCENT).roundToInt()} %",
            caption = "$missing without a category",
            meter = stats.categorizedShare,
            link = StatLink.UNCATEGORIZED.takeIf { missing > 0 },
        )
    }

    private fun inbox(state: DashboardUiStateModel): StatColumnModel {
        val inbox = state.inbox ?: return StatColumnModel(
            label = "Inbox",
            value = "Off",
            caption = "Switched off in the settings",
        )
        return StatColumnModel(
            label = "Inbox",
            value = inbox.count.toString(),
            caption = if (inbox.count == 0) "Nothing to file" else "Waiting to be filed",
            link = StatLink.INBOX,
        )
    }

    private fun storage(state: DashboardUiStateModel): StatColumnModel {
        return StatColumnModel(
            label = "Storage",
            value = FileSizes.format(bytes = state.stats.totalBytes),
            caption = "Kept in each of ${state.folders.size} library folders",
        )
    }
}
