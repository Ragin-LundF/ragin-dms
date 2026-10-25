package dms.ui.shell.palette

import dms.domain.document.model.DocumentIdModel
import dms.ui.ask.AskViewModel
import dms.ui.dashboard.DashboardViewModel
import dms.ui.library.LibraryViewModel
import dms.ui.model.Screen
import dms.ui.session.LibrarySession
import dms.ui.shell.palette.model.PaletteAction
import dms.ui.shell.palette.model.PaletteItemModel

/**
 * Runs what a palette line stands for, through the same view model functions as the buttons, so the
 * same confirmations, lanes and busy rules apply.
 */
class PaletteCommands(
    private val session: LibrarySession,
    private val dashboard: DashboardViewModel,
    private val library: LibraryViewModel,
    private val ask: AskViewModel,
    private val onScreenChange: (Screen) -> Unit,
    private val onOpenDocument: (DocumentIdModel) -> Unit,
) {

    fun run(item: PaletteItemModel) {
        when (item) {
            is PaletteItemModel.OpenDocument -> onOpenDocument(item.documentId)
            is PaletteItemModel.GoTo -> {
                library.show(selection = item.selection, query = "")
                onScreenChange(Screen.LIBRARY)
            }

            // An action that waits for its lane is never run; the palette shows why instead.
            is PaletteItemModel.Run -> if (item.disabledReason == null) run(action = item.action)
            is PaletteItemModel.AskLibrary -> {
                onScreenChange(Screen.ASK)
                ask.ask(question = item.question)
            }
        }
    }

    private fun run(action: PaletteAction) {
        when (action) {
            PaletteAction.SCAN -> dashboard.scan()
            PaletteAction.SYNC_ALL -> dashboard.syncToAllDestinations()
            PaletteAction.RETRY_FAILED -> session.retryFailedOperations()
            PaletteAction.DETECT_DATES -> session.detectDocumentDates()
            PaletteAction.ADD_FILES -> library.addFilesToTopLevel()
            PaletteAction.MERGE_ALL -> dashboard.mergeAllFolders()
            // The AI batches show their progress and results on the dashboard, so it comes to the front.
            PaletteAction.TAG_UNTAGGED -> onDashboard { dashboard.aiBatches.tagAll() }
            PaletteAction.CLASSIFY_UNCATEGORIZED -> onDashboard { dashboard.aiBatches.classifyAll() }
            PaletteAction.OPEN_DASHBOARD -> onScreenChange(Screen.DASHBOARD)
            PaletteAction.OPEN_TASKS -> onScreenChange(Screen.TASKS)
            PaletteAction.OPEN_LIBRARY -> onScreenChange(Screen.LIBRARY)
            PaletteAction.OPEN_SETTINGS -> onScreenChange(Screen.SETTINGS)
        }
    }

    private fun onDashboard(block: () -> Unit) {
        onScreenChange(Screen.DASHBOARD)
        block()
    }
}
