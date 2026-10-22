package dms.ui.session

import dms.application.common.OperationOutcome
import dms.application.library.model.OpenLibraryOutcome
import dms.application.library.model.ScanResultModel
import dms.application.port.state.model.PreferencesModel
import dms.domain.change.DetectedChange
import dms.ui.service.AppServices
import dms.ui.session.model.LibraryStatus
import dms.ui.session.model.OperationLane
import dms.ui.session.model.OperationProgressModel
import dms.ui.session.model.RunningOperationModel
import dms.ui.session.model.SessionStateModel
import dms.ui.session.model.UiMessageModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.time.Duration
import java.util.Locale
import java.util.concurrent.ConcurrentHashMap
import kotlin.coroutines.cancellation.CancellationException

/**
 * The open library as every screen sees it, and the one place that runs operations changing the
 * library. Each operation runs in a lane: while one runs, others in its lane are refused, so two of
 * them never write the same files at once, but an AI run and a change of the library run side by side.
 */
class LibrarySession(
    private val services: AppServices,
    private val scope: CoroutineScope,
) {

    private val mutableState = MutableStateFlow(value = SessionStateModel())
    private val mutableProgress = MutableStateFlow<Map<OperationLane, OperationProgressModel>>(value = emptyMap())
    private val running = ConcurrentHashMap<OperationLane, Job>()

    val state: StateFlow<SessionStateModel> = mutableState.asStateFlow()

    /**
     * How far the running operations are, by lane. Kept apart from [state], because it changes with
     * every file and screens that only need [state] should not recompute that often.
     */
    val progress: StateFlow<Map<OperationLane, OperationProgressModel>> = mutableProgress.asStateFlow()

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

    /** Stops the operation running in [lane] if it can be stopped safely; see [perform]. */
    fun cancel(lane: OperationLane) {
        val operation = mutableState.value.operations.firstOrNull { it.lane == lane && it.cancellable } ?: return
        log.info(text = "Cancelling: ${operation.label}")
        running[lane]?.cancel()
    }

    /** Switches adding new files during a scan on or off; it applies from the next scan. */
    fun setAddNewFilesWhileScanning(enabled: Boolean) {
        updatePreferences { it.copy(addNewFilesWhileScanning = enabled) }
    }

    /** Saves the preferences of this device; the one writer of them. */
    fun updatePreferences(change: (PreferencesModel) -> PreferencesModel) {
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
            report(message = UiMessageModel(text = "Found a date for $dated documents"))
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
     * Runs [block] as the one operation of its [lane]. Returns `false` and does nothing while another
     * operation runs in that lane. Only a [cancellable] operation can be stopped by the user: one
     * that, like a scan, leaves the library consistent wherever it stops.
     */
    fun perform(
        label: String,
        lane: OperationLane = OperationLane.LIBRARY,
        cancellable: Boolean = false,
        block: suspend () -> Unit,
    ): Boolean {
        val operation = RunningOperationModel(label = label, lane = lane, cancellable = cancellable)
        if (!claim(operation = operation)) {
            return false
        }
        val job = launchLogged(label = label, block = block)
        running[lane] = job
        job.invokeOnCompletion { cause -> finish(operation = operation, cancelled = cause is CancellationException) }
        return true
    }

    /**
     * Adds [operation] to the running ones unless its lane is busy. Retries when another lane changed
     * the state at the same moment, so a busy AI run never makes a library operation fail to start.
     */
    private fun claim(operation: RunningOperationModel): Boolean {
        while (true) {
            val current = mutableState.value
            if (current.busyIn(lane = operation.lane) != null) {
                return false
            }
            val update = current.copy(operations = current.operations + operation)
            if (mutableState.compareAndSet(expect = current, update = update)) {
                return true
            }
        }
    }

    /** Runs [block], logging how long it took, and reports a failure instead of throwing it. */
    private fun launchLogged(label: String, block: suspend () -> Unit): Job {
        log.info(text = "$label…")
        val started = services.library.clock.now()
        return scope.launch {
            val failure = runCatching { block() }.exceptionOrNull()
            if (failure is CancellationException) {
                throw failure
            }
            if (failure != null) {
                val detail = failure.message ?: failure::class.simpleName
                report(message = UiMessageModel(text = "$label failed: $detail", isError = true))
            } else {
                val seconds = Duration.between(started, services.library.clock.now()).toMillis() / MILLIS
                log.info(text = "$label: finished in ${"%.1f".format(Locale.ROOT, seconds)} s")
            }
        }
    }

    /** Frees the session after an operation; runs even when it was cancelled before it started. */
    private fun finish(operation: RunningOperationModel, cancelled: Boolean) {
        if (cancelled) {
            log.info(text = "${operation.label}: cancelled")
            report(message = UiMessageModel(text = "${operation.label}: cancelled"))
        }
        running.remove(operation.lane)
        mutableProgress.update { it - operation.lane }
        mutableState.update { it.copy(operations = it.operations - operation, revision = it.revision + 1) }
    }

    /**
     * Tells the screens that the running operation changed something they show, such as a new
     * suggestion under Open tasks, so they reload it now instead of when the operation ends.
     */
    fun changed() {
        mutableState.update { it.copy(revision = it.revision + 1) }
    }

    /**
     * Shows how far the operation running in [lane] is. Call inside [perform]; it is cleared when the
     * operation ends.
     */
    fun progress(progress: OperationProgressModel, lane: OperationLane = OperationLane.LIBRARY) {
        mutableProgress.update { it + (lane to progress) }
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
            progress(
                progress = OperationProgressModel(step = step, done = index, total = items.size, detail = name(item)),
            )
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
            is OpenLibraryOutcome.PrimaryFailed -> {
                val failure = outcome.failure
                log.error(text = "The primary folder failed: ${failure.reason} ${failure.detail.orEmpty()}".trim())
                LibraryStatus.PrimaryFailed(reason = failure.reason, detail = failure.detail)
            }

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
        collectInbox(result = result)
        if (adding) {
            adoptNewFiles(result = result)
        }
    }

    /**
     * Catalogs what arrived in the inbox of any folder and, with "Analyze automatically" on, lets the
     * AI suggest metadata for it in the background. Call inside [perform].
     */
    private suspend fun collectInbox(result: ScanResultModel) {
        val collected = services.organize.collectInbox(scan = result)
        collected.forEach { forget(change = it) }
        if (collected.isEmpty()) {
            return
        }
        log.info(text = "Inbox: ${collected.size} new documents")
        val inbox = services.library.catalog.value?.settings?.layout?.inbox ?: return
        if (inbox.autoAnalyze && inbox.aiAnalysis) {
            analyzeInbox()
        }
    }

    /** Lets the AI suggest metadata for every inbox document without one, in the AI lane. */
    fun analyzeInbox() {
        val inbox = services.library.catalog.value?.settings?.layout?.inbox ?: return
        perform(label = "Analyzing the inbox", lane = OperationLane.AI, cancellable = true) {
            services.ai.classifyAllNew(scope = inbox.path).collect { progress ->
                val step = OperationProgressModel(
                    step = "Analyzing the inbox",
                    done = progress.done,
                    total = progress.total,
                )
                progress(progress = step, lane = OperationLane.AI)
                changed()
            }
        }
    }

    /**
     * Scans a few seconds after something changed in the inbox of any folder, while the inbox is on.
     * Only a hint: the scan decides what changed. Call once when the session starts.
     */
    @OptIn(ExperimentalCoroutinesApi::class, FlowPreview::class)
    fun watchInbox() {
        scope.launch {
            services.library.catalog
                .map { catalog -> catalog?.settings?.layout?.inbox?.takeIf { it.enabled }?.path }
                .distinctUntilChanged()
                .flatMapLatest { path ->
                    val folders = state.value.folders?.folders.orEmpty()
                    if (path == null || folders.isEmpty()) {
                        emptyFlow()
                    } else {
                        services.organize.folderWatcher.changes(folders = folders, path = path)
                    }
                }
                .debounce(timeoutMillis = INBOX_DEBOUNCE_MILLIS)
                .collect {
                    if (state.value.status is LibraryStatus.Open) {
                        scan()
                    }
                }
        }
    }

    private suspend fun adoptNewFiles(result: ScanResultModel) {
        // A copy of a catalogued document, or of another new file, would only become a second document.
        val known = services.library.catalog.value?.documents?.values.orEmpty().map { it.contentHash }.toSet()
        val newFiles = result.changes.filterIsInstance<DetectedChange.NewFile>()
            .filter { it.contentHash != null && it.contentHash !in known }
            .distinctBy { it.contentHash }
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

    fun report(message: UiMessageModel) {
        mutableState.update { it.copy(message = message) }
    }

    fun clearMessage() {
        mutableState.update { it.copy(message = null) }
    }

    private companion object {
        const val MILLIS = 1_000.0
        const val INBOX_DEBOUNCE_MILLIS = 3_000L
    }
}
