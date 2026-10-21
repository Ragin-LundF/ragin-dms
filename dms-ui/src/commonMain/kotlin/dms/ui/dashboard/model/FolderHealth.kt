package dms.ui.dashboard.model

/** What the dashboard knows about a library folder. */
enum class FolderHealth(val label: String) {
    NOT_SCANNED(label = "not scanned yet"),
    REACHABLE(label = "reachable"),
    OFFLINE(label = "offline"),
    FOREIGN(label = "holds another library, left alone"),
}
