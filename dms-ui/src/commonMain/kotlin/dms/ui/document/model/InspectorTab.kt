package dms.ui.document.model

/** The tabs of the inspector, in the order they show. */
enum class InspectorTab(val label: String) {
    DETAILS(label = "Details"),
    NOTES_AND_LINKS(label = "Notes & links"),
    PAGES(label = "Pages"),
    HISTORY(label = "History"),
}
