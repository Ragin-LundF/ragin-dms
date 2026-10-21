package dms.ui.dashboard.model

/** The kinds of open tasks, in the order the dashboard lists them. */
enum class TaskGroup(val label: String) {
    SUGGESTIONS(label = "Suggestions"),
    NEW_FILES(label = "New files"),
    MISSING(label = "Missing files"),
    MODIFIED(label = "Changed outside"),
    MOVED(label = "Moved outside"),
    CONFLICTS(label = "Conflicts"),
    CATALOG_MERGE(label = "Catalog merge"),
    FAILED_OPERATIONS(label = "Failed operations"),
}
