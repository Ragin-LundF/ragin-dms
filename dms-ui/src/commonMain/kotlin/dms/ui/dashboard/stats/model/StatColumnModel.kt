package dms.ui.dashboard.stats.model

/**
 * One figure of the stat strip: what it counts, the value, a caption, and optionally a [meter] from
 * 0 to 1, a [trend] to draw as a sparkline and the [link] a click opens.
 */
data class StatColumnModel(
    val label: String,
    val value: String,
    val caption: String,
    val meter: Float? = null,
    val trend: List<Int> = emptyList(),
    val link: StatLink? = null,
)
