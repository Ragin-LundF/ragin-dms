package dms.ui.session

import dms.application.common.OperationOutcome
import dms.application.library.model.OpenLibraryOutcome
import dms.application.library.model.ScanResult
import dms.application.port.state.model.Preferences
import dms.domain.change.DetectedChange
import dms.ui.service.AppServices
import dms.ui.session.model.LibraryStatus
import dms.ui.session.model.OperationProgress
import dms.ui.session.model.SessionState
import dms.ui.session.model.UiMessage
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.time.Duration
import java.util.Locale
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
    private val mutableProgress = MutableStateFlow<OperationProgress?>(value = null)
    private var running: Job? = null

    val state: StateFlow<SessionState> = mutableState.asStateFlow()

    /**
     * How far the running operation is. Kept apart from [state], because it changes with every file
     * and screens that only need [state] should not recompute that often.
     */
    val progress: StateFlow<OperationProgress?> = mutableProgress.asStateFlow()

    val log: SessionLog = SessionLog(clock = services.library.clock)

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
        perform(label = "Scanning the library folders", cancellable = true) { rescan() }
    }

    /** Stops the running operation if it can be stopped safely; see [perform]. */
    fun cancel() {
        if (mutableState.value.cancellable) {
            log.info(text = "Cancelling: ${mutableState.value.busy}")
            running?.cancel()
        }
    }

    /** Switches adding new files during a scan on or off; it applies from the next scan. */
    fun setAddNewFilesWhileScanning(enabled: Boolean) {
        updatePreferences { it.copy(addNewFilesWhileScanning = enabled) }
    }

    /** Saves the preferences of this device; the one writer of them. */
    fun updatePreferences(change: (Preferences) -> Preferences) {
        perform(label = "Saving the preferences") {
            val preferences = change(mutableState.value.preferences)
            services.library.settings.savePreferences(preferences = preferences)
            mutableState.update { it.copy(preferences = preferences) }
        }
    }

    /** Gives documents without a date the one found in their file name or creation date. */
    fun detectDocumentDates() {
        perform(label = "Detecting document dates") {
            val dated = services.library.detectDocumentDates()
            report(message = UiMessage(text = "Found a date for $dated documents"))
        }
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
     * nothing while another operation runs. Only a [cancellable] operation can be stopped by the
     * user: one that, like a scan, leaves the library consistent wherever it stops.
     */
    fun perform(label: String, cancellable: Boolean = false, block: suspend () -> Unit): Boolean {
        val current = mutableState.value
        val claimed = current.busy == null &&
            mutableState.compareAndSet(expect = current, update = current.copy(busy = label, cancellable = cancellable))
        if (!claimed) {
            return false
        }
        log.info(text = "$label…")
        val started = services.library.clock.now()
        val job = scope.launch {
            val failure = runCatching { block() }.exceptionOrNull()
            if (failure is CancellationException) {
                throw failure
            }
            if (failure != null) {
                val detail = failure.message ?: failure::class.simpleName
                report(message = UiMessage(text = "$label failed: $detail", isError = true))
            } else {
                val seconds = Duration.between(started, services.library.clock.now()).toMillis() / MILLIS
                log.info(text = "$label: finished in ${"%.1f".format(Locale.ROOT, seconds)} s")
            }
        }
        job.invokeOnCompletion { cause -> finish(label = label, cancelled = cause is CancellationException) }
        running = job
        return true
    }

    /** Frees the session after an operation; runs even when it was cancelled before it started. */
    private fun finish(label: String, cancelled: Boolean) {
        if (cancelled) {
            log.info(text = "$label: cancelled")
            report(message = UiMessage(text = "$label: cancelled"))
        }
        mutableProgress.value = null
        mutableState.update { it.copy(busy = null, cancellable = false, revision = it.revision + 1) }
    }

    /**
     * Tells the screens that the running operation changed something they show, such as a new
     * suggestion under Open tasks, so they reload it now instead of when the operation ends.
     */
    fun changed() {
        mutableState.update { it.copy(revision = it.revision + 1) }
    }

    /** Shows how far the running operation is. Call inside [perform]; it is cleared when it ends. */
    fun progress(progress: OperationProgress) {
        mutableProgress.value = progress
    }

    /**
     * Runs [action] for every item, one after the other, with progress and a log line each. Returns
     * how many succeeded. Call inside [perform].
     */
    suspend fun <T> forEachWithProgress(
        step: String,
        items: List<T>,
        name: (T) -> String,
        action: suspend (T) -> Boolean,
    ): Int {
        var succeeded = 0
        items.forEachIndexed { index, item ->
            progress(progress = OperationProgress(step = step, done = index, total = items.size, detail = name(item)))
            if (action(item)) {
                succeeded++
                log.info(text = "$step: ${name(item)}")
            } else {
                log.error(text = "$step failed: ${name(item)}")
            }
        }
        return succeeded
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
        val preferences = services.library.settings.loadPreferences()
        mutableState.update {
            it.copy(status = status, folders = folders, preferences = preferences, lastScan = null, lastScanAt = null)
        }
        if (status is LibraryStatus.Open) {
            rescan()
        }
    }

    /**
     * Scans every folder; the dashboard turns the result into tasks. With the preference on, new files
     * join the library during the scan, and files only the sync folders have are adopted after it.
     * Call inside [perform].
     */
    suspend fun rescan() {
        val adding = mutableState.value.preferences.addNewFilesWhileScanning
        val reporter = ScanReporter(log = log) { progress(progress = it) }
        val result = services.library.scanLibrary(addNewFiles = adding, onProgress = reporter::report) ?: return
        val now = services.library.clock.now()
        mutableState.update { it.copy(lastScan = result, lastScanAt = now) }
        log.info(text = "Scan finished: ${result.changes.size} open changes")
        result.failedFolders.keys.forEach { log.error(text = "Folder could not be scanned: ${it.value}") }
        if (adding) {
            adoptNewFiles(result = result)
        }
    }

    private suspend fun adoptNewFiles(result: ScanResult) {
        val newFiles = result.changes.filterIsInstance<DetectedChange.NewFile>().filter { it.contentHash != null }
        if (newFiles.isEmpty()) {
            return
        }
        forEachWithProgress(step = "Adding new files", items = newFiles, name = { it.path.value }) { change ->
            val done = services.documents.adoptNewFile(change = change) is OperationOutcome.Done
            if (done) {
                forget(change = change)
            }
            done
        }
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

    private companion object {
        const val MILLIS = 1_000.0
    }
}
