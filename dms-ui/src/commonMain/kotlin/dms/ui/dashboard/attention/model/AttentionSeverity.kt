package dms.ui.dashboard.attention.model

/**
 * How urgent a row of "Needs your attention" is, most urgent first. The stripe colour of a row
 * always comes with its [label], so colour is never the only signal.
 */
enum class AttentionSeverity(val label: String) {
    PASSED(label = "Date passed"),
    SOON(label = "Decide soon"),
    INFO(label = "To do"),
}
