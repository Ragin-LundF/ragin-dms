package dms.ui.dashboard.organize.model

/** The inbox on the dashboard: where it is, how many documents wait in it, and whether the AI analyzes them. */
data class InboxSummaryModel(
    val path: String,
    val count: Int,
    val aiAnalysis: Boolean,
)
