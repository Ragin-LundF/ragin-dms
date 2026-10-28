package dms.ui.shell.palette.model

import dms.ui.session.model.OperationLane

/**
 * The actions the palette offers: the ones of the menu bar, with the key of their shortcut, if any, and
 * the [lane] they run in; `null` for going to a screen, which never waits.
 */
enum class PaletteAction(
    val label: String,
    val shortcutKey: String?,
    val lane: OperationLane?,
    val shift: Boolean = false,
) {
    SCAN(label = "Scan now", shortcutKey = "R", lane = OperationLane.LIBRARY),
    SYNC_ALL(label = "Sync to all destinations", shortcutKey = "S", lane = OperationLane.LIBRARY, shift = true),
    MERGE_ALL(label = "Merge all folders", shortcutKey = "M", lane = OperationLane.LIBRARY, shift = true),
    RETRY_FAILED(label = "Retry failed operations", shortcutKey = null, lane = OperationLane.LIBRARY),
    DETECT_DATES(label = "Detect missing document dates", shortcutKey = null, lane = OperationLane.LIBRARY),
    ADD_FILES(label = "Add files…", shortcutKey = null, lane = OperationLane.LIBRARY),
    TAG_UNTAGGED(label = "Tag untagged documents with AI", shortcutKey = null, lane = OperationLane.AI),
    CLASSIFY_UNCATEGORIZED(
        label = "Classify uncategorized documents with AI",
        shortcutKey = null,
        lane = OperationLane.AI,
    ),
    CONVERT_TO_REAL_PDF(label = "Convert the open PDF to a real PDF", shortcutKey = null, lane = OperationLane.AI),
    OPEN_DASHBOARD(label = "Go to Dashboard", shortcutKey = "1", lane = null),
    OPEN_TASKS(label = "Go to Open tasks", shortcutKey = "2", lane = null),
    OPEN_LIBRARY(label = "Go to Library", shortcutKey = "3", lane = null),
    OPEN_SETTINGS(label = "Go to Settings", shortcutKey = ",", lane = null),
}
