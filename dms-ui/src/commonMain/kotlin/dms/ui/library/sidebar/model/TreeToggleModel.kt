package dms.ui.library.sidebar.model

/**
 * The +/- of a tree entry. [expanded] is `null` for an entry without children, which keeps the
 * space of the symbol so names line up.
 */
data class TreeToggleModel(
    val expanded: Boolean?,
    val onToggle: () -> Unit,
)
