package dms.ui.components.chart.model

/** One value of a single-series chart; [id] identifies what a click on it opens. */
data class ChartPoint(
    val id: String,
    val label: String,
    val value: Int,
)
