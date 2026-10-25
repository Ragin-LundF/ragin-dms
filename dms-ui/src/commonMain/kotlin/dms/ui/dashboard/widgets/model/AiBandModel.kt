package dms.ui.dashboard.widgets.model

/**
 * The AI band: its [headline], and the labels of its two buttons, `null` while there is nothing for
 * that button to do. [busyReason] names the AI run that keeps both disabled.
 */
data class AiBandModel(
    val headline: String,
    val classifyLabel: String?,
    val tagLabel: String?,
    val busyReason: String?,
)
