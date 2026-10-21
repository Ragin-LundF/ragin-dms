package dms.ui.session

import dms.application.library.model.OpenLibraryOutcome
import dms.domain.change.DetectedChange
import dms.ui.service.AppServices
import dms.ui.session.model.LibraryStatus
import dms.ui.session.model.SessionState
import dms.ui.session.model.UiMessage
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlin.coroutines.cancellation.CancellationException

/**
 * The open library as every screen sees it, and the one place that runs operations changing the
 * library. Operations run one after the other: while one runs, others are refused, so two of them
 * never write the same files at once.
 */
class LibrarySession(
    private val services: AppServices,
    private val scope: CoroutineScope,
) {

    private val mutableState = MutableStateFlow(value = SessionState())

    val state: StateFlow<SessionState> = mutableState.asStateFlow()

    /** Opens the library, merging the catalogs of all folders, and scans it. */
    fun open() {
        perform(label = "Opening the library") {
            val opened = runCatching { reopen() }
            val failure = opened.exceptionOrNull()
            if (failure is CancellationException) {
                throw failure
            }
            if (failure != null) {
                mutableState.update { it.copy(status = LibraryStatus.Failed(detail = failure.message)) }
            }
        }
    }

    fun scan() {
        perform(label = "Scanning the library folders") { rescan() }
    }

    fun syncToAllDestinations() {
        perform(label = "Syncing to all destinations") {
            val outcome = services.sync.syncFromPrimary()
            report(message = OutcomeMessages.sync(outcome = outcome, done = "Synced to all destinations"))
            rescan()
        }
    }

    fun mergeAllFolders() {
        perform(label = "Merging all folders") {
            val outcome = services.sync.mergeAllFolders()
            report(message = OutcomeMessages.sync(outcome = outcome, done = "Merged all folders"))
            rescan()
        }
    }

    fun retryFailedOperations() {
        perform(label = "Retrying failed operations") {
            report(message = OutcomeMessages.retry(report = services.sync.retryFailedOperations()))
        }
    }

    /**
     * Runs [block] as the one operation that may change the library. Returns `false` and does
     * nothing while another operation runs.
     */
    fun perform(label: String, block: suspend () -> Unit): Boolean {
        val current = mutableState.value
        val claimed = current.busy == null &&
            mutableState.compareAndSet(expect = current, update = current.copy(busy = label))
        if (!claimed) {
            return false
        }
        scope.launch {
            try {
                val failure = runCatching { block() }.exceptionOrNull()
                if (failure is CancellationException) {
                    throw failure
                }
                if (failure != null) {
                    val detail = failure.message ?: failure::class.simpleName
                    report(message = UiMessage(text = "$label failed: $detail", isError = true))
                }
            } finally {
                mutableState.update { it.copy(busy = null, revision = it.revision + 1) }
            }
        }
        return true
    }

    /** Opens the library again, for example after its folders changed. Call inside [perform]. */
    suspend fun reopen() {
        val folders = services.library.settings.loadLibraryFolders()
        val status = when (val outcome = services.library.openLibrary()) {
            OpenLibraryOutcome.NotConfigured -> LibraryStatus.NotConfigured
            is OpenLibraryOutcome.PrimaryFailed -> LibraryStatus.PrimaryFailed(reason = outcome.failure.reason)
            is OpenLibraryOutcome.Opened -> LibraryStatus.Open(
                foreignFolders = outcome.foreignFolders,
                unreachableFolders = outcome.unreachableFolders,
            )
        }
        mutableState.update { it.copy(status = status, folders = folders, lastScan = null, lastScanAt = null) }
        if (status is LibraryStatus.Open) {
            rescan()
        }
    }

    /** Scans every folder; the dashboard turns the result into tasks. Call inside [perform]. */
    suspend fun rescan() {
        val result = services.library.scanLibrary() ?: return
        val now = services.library.clock.now()
        mutableState.update { it.copy(lastScan = result, lastScanAt = now) }
    }

    /** Removes a change the user resolved from the last scan, so its task disappears without a new scan. */
    fun forget(change: DetectedChange) {
        mutableState.update { state ->
            state.copy(lastScan = state.lastScan?.let { scan -> scan.copy(changes = scan.changes - change) })
        }
    }

    fun report(message: UiMessage) {
        mutableState.update { it.copy(message = message) }
    }

    fun clearMessage() {
        mutableState.update { it.copy(message = null) }
    }
}
