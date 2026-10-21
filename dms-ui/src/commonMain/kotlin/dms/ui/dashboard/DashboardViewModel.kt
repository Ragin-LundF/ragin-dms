package dms.ui.dashboard

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dms.application.common.OperationOutcome
import dms.application.library.model.Dashboard
import dms.application.library.model.FolderState
import dms.application.resolution.model.MoveDecision
import dms.domain.change.DetectedChange
import dms.domain.document.DocumentId
import dms.domain.library.FolderRole
import dms.domain.library.LibraryFolderId
import dms.domain.task.OpenTask
import dms.domain.task.TaskKey
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ConfirmationRequest
import dms.ui.dashboard.model.DashboardUiState
import dms.ui.dashboard.model.FolderHealth
import dms.ui.dashboard.model.FolderStatus
import dms.ui.dashboard.model.TaskGroup
import dms.ui.dashboard.model.TaskGroupState
import dms.ui.dashboard.model.TaskRowState
import dms.ui.dashboard.stats.LibraryStatistics
import dms.ui.dashboard.task.TaskActions
import dms.ui.dashboard.task.TaskContext
import dms.ui.dashboard.task.TaskGroups
import dms.ui.dashboard.task.TaskHandler
import dms.ui.dashboard.task.TaskTexts
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.LibraryStatus
import dms.ui.session.model.SessionState
import dms.ui.session.model.UiMessage
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import java.time.ZoneId

/**
 * The home screen: the open tasks with their actions, the folders and the recent activity. Every
 * action runs through the session, one at a time, and a task disappears only when its use case
 * succeeded or the user dismissed it.
 */
class DashboardViewModel(
    private val services: AppServices,
    private val session: LibrarySession,
) : ViewModel(), TaskHandler {

    private val dashboard = MutableStateFlow<Dashboard?>(value = null)
    private val filter = MutableStateFlow<TaskGroup?>(value = null)

    val dialogs: DialogController = DialogController()

    val state: StateFlow<DashboardUiState> = combine(
        flow = dashboard,
        flow2 = session.state,
        flow3 = filter,
    ) { built, sessionState, group ->
        toUiState(dashboard = built, sessionState = sessionState, filter = group)
    }.stateIn(scope = viewModelScope, started = SharingStarted.Eagerly, initialValue = DashboardUiState())

    init {
        viewModelScope.launch {
            val scans = session.state.map { it.lastScan to it.revision }.distinctUntilChanged()
            combine(flow = scans, flow2 = services.library.catalog) { (scan, _), _ -> scan }
                .collectLatest { scan -> dashboard.value = services.library.buildDashboard(scan = scan) }
        }
    }

    fun setFilter(group: TaskGroup?) {
        filter.value = group
    }

    fun scan() {
        session.scan()
    }

    fun syncToAllDestinations() {
        session.syncToAllDestinations()
    }

    fun mergeAllFolders() {
        session.mergeAllFolders()
    }

    override fun adopt(change: DetectedChange.NewFile) {
        act(label = "Adopting ${change.path}", resolves = change, done = "Adopted ${change.path}") {
            services.documents.adoptNewFile(change = change)
        }
    }

    override fun restore(change: DetectedChange.MissingFile) {
        act(label = "Restoring ${change.path}", resolves = change, done = "Restored ${change.path}") {
            services.documents.restoreDocument(change = change)
        }
    }

    override fun deleteEverywhere(change: DetectedChange.MissingFile) {
        val folders = session.state.value.folders?.folders.orEmpty()
        dialogs.ask(
            request = ConfirmationRequest(
                title = "Delete everywhere",
                message = "The file goes to the trash in every library folder and leaves the catalog.",
                affected = folders.map { "${it.displayName}: ${change.path}" },
                confirmLabel = "Move to trash",
            ),
        ) {
            act(label = "Deleting ${change.path}", resolves = change, done = "Moved ${change.path} to the trash") {
                services.documents.trashDocument(documentId = change.documentId, confirmed = true)
            }
        }
    }

    override fun keepVersion(change: DetectedChange, sourceFolderId: LibraryFolderId) {
        val folders = session.state.value.folders?.folders.orEmpty()
        val source = folders.firstOrNull { it.id == sourceFolderId }?.displayName ?: sourceFolderId.value
        dialogs.ask(
            request = ConfirmationRequest(
                title = "Keep the version of $source",
                message = "Every other version of this file goes to the trash and is replaced by the one in $source.",
                affected = folders.filter { it.id != sourceFolderId }.map { "${it.displayName}: ${change.path}" },
                confirmLabel = "Keep this version",
            ),
        ) {
            act(label = "Keeping the version of $source", resolves = change, done = "Kept the version of $source") {
                services.sync.keepVersion(path = change.path, sourceFolderId = sourceFolderId, confirmed = true)
            }
        }
    }

    override fun keepBoth(change: DetectedChange.ContentConflict, renamedIn: LibraryFolderId) {
        act(label = "Keeping both versions", resolves = change, done = "Kept both versions of ${change.path}") {
            services.sync.keepBothVersions(path = change.path, folderId = renamedIn)
        }
    }

    override fun resolveMove(change: DetectedChange.MovedFile, decision: MoveDecision) {
        val done = when (decision) {
            MoveDecision.APPLY -> "Moved ${change.previousPath} to ${change.path} everywhere"
            MoveDecision.UNDO -> "Moved ${change.path} back to ${change.previousPath}"
        }
        act(label = "Resolving the move of ${change.path}", resolves = change, done = done) {
            services.sync.resolveMove(change = change, decision = decision)
        }
    }

    override fun reject(documentId: DocumentId) {
        session.perform(label = "Rejecting the suggestion") {
            services.sync.rejectSuggestion(documentId = documentId)
        }
    }

    override fun retry() {
        session.retryFailedOperations()
    }

    override fun dismiss(key: TaskKey) {
        session.perform(label = "Dismissing the task") {
            services.sync.dismissTask(key = key)
        }
    }

    /** Adopts every listed new file, one after the other; can be stopped between two files. */
    fun adoptAll(changes: List<DetectedChange.NewFile>) {
        session.perform(label = "Adopting ${changes.size} new files", cancellable = true) {
            val adopted = session.forEachWithProgress(step = "Adopting", items = changes, name = { it.path.value }) {
                resolved(change = it, outcome = services.documents.adoptNewFile(change = it))
            }
            session.report(message = countMessage(verb = "Adopted", done = adopted, total = changes.size))
        }
    }

    /** Restores every listed missing file from a folder that still has it. */
    fun restoreAll(changes: List<DetectedChange.MissingFile>) {
        session.perform(label = "Restoring ${changes.size} files") {
            val restored = session.forEachWithProgress(step = "Restoring", items = changes, name = { it.path.value }) {
                resolved(change = it, outcome = services.documents.restoreDocument(change = it))
            }
            session.report(message = countMessage(verb = "Restored", done = restored, total = changes.size))
        }
    }

    /** Forgets [change] when [outcome] resolved it; returns whether it did. */
    private fun resolved(change: DetectedChange, outcome: OperationOutcome): Boolean {
        val done = outcome is OperationOutcome.Done
        if (done) {
            session.forget(change = change)
        }
        return done
    }

    private fun act(label: String, resolves: DetectedChange, done: String, action: suspend () -> OperationOutcome) {
        session.perform(label = label) {
            val outcome = action()
            if (outcome is OperationOutcome.Done) {
                session.forget(change = resolves)
            }
            session.report(message = OutcomeMessages.of(outcome = outcome, done = done))
        }
    }

    private fun countMessage(verb: String, done: Int, total: Int): UiMessage {
        return UiMessage(text = "$verb $done of $total files", isError = done < total)
    }

    private fun toUiState(
        dashboard: Dashboard?,
        sessionState: SessionState,
        filter: TaskGroup?,
    ): DashboardUiState {
        val context = TaskContext(folders = sessionState.folders, catalog = services.library.catalog.value)
        val byGroup = dashboard?.tasks.orEmpty().groupBy { TaskGroups.of(task = it) }
        val foreign = (sessionState.status as? LibraryStatus.Open)?.foreignFolders.orEmpty()
        return DashboardUiState(
            folders = dashboard?.folders.orEmpty().map { toFolderStatus(state = it, foreign = foreign) },
            documentCount = dashboard?.documentCount ?: 0,
            uncategorizedCount = dashboard?.uncategorizedCount ?: 0,
            groups = TaskGroup.entries.mapNotNull { group ->
                byGroup[group]?.let { tasks -> toGroupState(group = group, tasks = tasks, context = context) }
            },
            filter = filter,
            activity = dashboard?.recentActivity.orEmpty(),
            lastScanAt = sessionState.lastScanAt,
            busy = sessionState.busy,
            stats = LibraryStatistics.of(
                catalog = services.library.catalog.value,
                now = services.library.clock.now(),
                zone = ZoneId.systemDefault(),
            ),
        )
    }

    private fun toGroupState(group: TaskGroup, tasks: List<OpenTask>, context: TaskContext): TaskGroupState {
        val rows = tasks.map { task ->
            TaskRowState(
                key = task.key,
                title = TaskTexts.title(task = task, context = context),
                detail = TaskTexts.detail(task = task, context = context),
                buttons = TaskActions.buttonsFor(task = task, context = context, handler = this),
            )
        }
        return TaskGroupState(group = group, rows = rows, bulkAction = bulkAction(group = group, tasks = tasks))
    }

    private fun bulkAction(group: TaskGroup, tasks: List<OpenTask>): ActionButton? {
        return when (group) {
            TaskGroup.NEW_FILES -> {
                val changes = tasks.filterIsInstance<OpenTask.AdoptNewFile>().map { it.change }
                val readable = changes.filter { it.contentHash != null }
                ActionButton(label = "Adopt all (${readable.size})") { adoptAll(changes = readable) }
            }

            TaskGroup.MISSING -> {
                val changes = tasks.filterIsInstance<OpenTask.RestoreOrDelete>().map { it.change }
                val restorable = changes.filter { it.presentIn.isNotEmpty() }
                ActionButton(label = "Restore all (${restorable.size})") { restoreAll(changes = restorable) }
            }

            else -> null
        }
    }

    private fun toFolderStatus(state: FolderState, foreign: Set<LibraryFolderId>): FolderStatus {
        val health = when {
            state.folder.id in foreign -> FolderHealth.FOREIGN
            state.reachable == null -> FolderHealth.NOT_SCANNED
            state.reachable == true -> FolderHealth.REACHABLE
            else -> FolderHealth.OFFLINE
        }
        return FolderStatus(
            name = state.folder.displayName,
            location = state.folder.location.value,
            isPrimary = state.folder.role == FolderRole.PRIMARY,
            health = health,
            pendingOperations = state.pendingOperations,
        )
    }
}
