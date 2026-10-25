package dms.ui.dashboard

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dms.application.common.OperationOutcome
import dms.application.library.model.DashboardModel
import dms.application.library.model.FolderStateModel
import dms.application.library.model.outcome.DashboardOutcome
import dms.application.resolution.model.MoveDecision
import dms.domain.catalog.model.CatalogModel
import dms.domain.change.DetectedChange
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.FolderRole
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.task.OpenTask
import dms.domain.task.model.TaskKeyModel
import dms.ui.PlatformActions
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ConfirmationRequestModel
import dms.ui.dashboard.bulk.BulkActions
import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.dashboard.model.FolderHealth
import dms.ui.dashboard.model.FolderStatusModel
import dms.ui.dashboard.model.TaskGroup
import dms.ui.dashboard.model.TaskGroupStateModel
import dms.ui.dashboard.model.TaskRowStateModel
import dms.ui.dashboard.organize.OrganizeTasks
import dms.ui.dashboard.organize.model.InboxSummaryModel
import dms.ui.dashboard.organize.model.UpcomingRowModel
import dms.ui.dashboard.stats.LibraryStatistics
import dms.ui.dashboard.task.TaskActions
import dms.ui.dashboard.task.TaskContext
import dms.ui.dashboard.task.TaskGroups
import dms.ui.dashboard.task.TaskHandler
import dms.ui.dashboard.task.TaskTexts
import dms.ui.preview.ThumbnailLoader
import dms.ui.review.ReviewController
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.LibraryStatus
import dms.ui.session.model.SessionStateModel
import dms.ui.session.model.UiMessageModel
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
    private val platform: PlatformActions,
) : ViewModel(), TaskHandler {

    private val dashboard = MutableStateFlow<DashboardModel?>(value = null)
    private val filter = MutableStateFlow<TaskGroup?>(value = null)

    val dialogs: DialogController = DialogController()
    val bulk: BulkActions = BulkActions(services = services, session = session, dialogs = dialogs)
    val review: ReviewController = ReviewController(services = services, session = session)
    val aiBatches: AiBatches = AiBatches(services = services, session = session)
    val thumbnails: ThumbnailLoader = ThumbnailLoader(services = services, platform = platform)
    val organize: OrganizeTasks = OrganizeTasks(
        services = services,
        session = session,
        dialogs = dialogs,
        platform = platform,
        scope = viewModelScope,
    )

    val state: StateFlow<DashboardUiStateModel> = combine(
        flow = dashboard,
        flow2 = session.state,
        flow3 = filter,
    ) { built, sessionState, group ->
        toUiState(dashboard = built, sessionState = sessionState, filter = group)
    }.stateIn(scope = viewModelScope, started = SharingStarted.Eagerly, initialValue = DashboardUiStateModel())

    init {
        viewModelScope.launch {
            val scans = session.state.map { it.lastScan to it.revision }.distinctUntilChanged()
            combine(flow = scans, flow2 = services.library.catalog) { (scan, _), _ -> scan }
                .collectLatest { scan ->
                    dashboard.value = when (val outcome = services.library.buildDashboard(scan = scan)) {
                        is DashboardOutcome.Built -> outcome.dashboard
                        DashboardOutcome.LibraryNotOpen -> null
                    }
                }
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

    fun analyzeInbox() {
        session.analyzeInbox()
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

    override fun trashNewFile(change: DetectedChange.NewFile) {
        val folders = session.state.value.folders?.folders.orEmpty().filter { it.id in change.presentIn }
        dialogs.ask(
            request = ConfirmationRequestModel(
                title = "Move to trash",
                message = "The file is not in the library. It goes to the trash in the folders that have it.",
                affected = folders.map { "${it.displayName}: ${change.path}" },
                confirmLabel = "Move to trash",
            ),
        ) {
            act(label = "Trashing ${change.path}", resolves = change, done = "Moved ${change.path} to the trash") {
                services.documents.trashNewFile(change = change, confirmed = true)
            }
        }
    }

    override fun trashDuplicateFile(change: DetectedChange.NewFile) {
        val original = services.library.catalog.value?.documents?.values
            ?.firstOrNull { it.contentHash == change.contentHash && it.size == change.size } ?: return
        val folders = session.state.value.folders?.folders.orEmpty().filter { it.id in change.presentIn }
        dialogs.ask(
            request = ConfirmationRequestModel(
                title = "Move copy to trash",
                message = "The file has exactly the same size and SHA-256 checksum as ${original.path}. " +
                    "Right before it goes to the trash it is compared with it byte by byte once more, in each folder.",
                affected = folders.map { "${it.displayName}: ${change.path}" },
                confirmLabel = "Move to trash",
            ),
        ) {
            val done = "Moved the copy ${change.path} to the trash"
            act(label = "Trashing ${change.path}", resolves = change, done = done) {
                services.documents.trashDuplicateFile(change = change, confirmed = true)
            }
        }
    }

    override fun keepDuplicate(task: OpenTask.ResolveDuplicates, keep: DocumentIdModel) {
        val documents = services.library.catalog.value?.documents.orEmpty()
        val kept = documents[keep] ?: return
        val copies = task.documentIds.filter { it != keep }.mapNotNull { documents[it] }
        val folders = session.state.value.folders?.folders.orEmpty()
        dialogs.ask(
            request = ConfirmationRequestModel(
                title = "Keep ${kept.path}",
                message = "The other copies have exactly the same size and SHA-256 checksum. Right before they go " +
                    "to the trash in every folder, each is compared byte by byte once more. Their categories, tags " +
                    "and title move to ${kept.path.fileName} where it has none.",
                affected = copies.flatMap { copy -> folders.map { "${it.displayName}: ${copy.path}" } },
                confirmLabel = "Move copies to trash",
            ),
        ) {
            session.perform(label = "Removing copies of ${kept.path}") {
                val outcome = services.sync.mergeDuplicates(keep = keep, confirmed = true)
                val done = "Kept ${kept.path}; moved ${copies.size} copies to the trash"
                session.report(message = OutcomeMessages.of(outcome = outcome, done = done))
            }
        }
    }

    override fun deleteEverywhere(change: DetectedChange.MissingFile) {
        val folders = session.state.value.folders?.folders.orEmpty()
        dialogs.ask(
            request = ConfirmationRequestModel(
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

    override fun keepVersion(change: DetectedChange, sourceFolderId: LibraryFolderIdModel) {
        val folders = session.state.value.folders?.folders.orEmpty()
        val source = folders.firstOrNull { it.id == sourceFolderId }?.displayName ?: sourceFolderId.value
        dialogs.ask(
            request = ConfirmationRequestModel(
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

    override fun keepBoth(change: DetectedChange.ContentConflict, renamedIn: LibraryFolderIdModel) {
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

    override fun review(suggestion: ClassificationSuggestionModel) {
        review.open(suggestion = suggestion)
    }

    override fun reject(documentId: DocumentIdModel) {
        session.perform(label = "Rejecting the suggestion") {
            services.ai.rejectSuggestion(documentId = documentId)
        }
    }

    override fun retry() {
        session.retryFailedOperations()
    }

    override fun open(folderId: LibraryFolderIdModel, path: RelativePathModel) {
        val folder = session.state.value.folders?.find(id = folderId) ?: return
        if (!platform.openFile(location = platform.locate(folder = folder.location, path = path))) {
            session.report(
                message = UiMessageModel(text = "No application could open ${path.fileName}", isError = true),
            )
        }
    }

    /** Opens a catalogued document from the primary folder, for example while reviewing a suggestion. */
    fun openDocument(documentId: DocumentIdModel) {
        val path = services.library.catalog.value?.documents?.get(documentId)?.path ?: return
        val primary = session.state.value.folders?.primary ?: return
        open(folderId = primary.id, path = path)
    }

    override fun dismiss(key: TaskKeyModel) {
        session.perform(label = "Dismissing the task") {
            services.sync.dismissTask(key = key)
        }
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

    /** Documents the AI can read that match [wanted] and have no suggestion waiting. */
    private fun aiCandidates(dashboard: DashboardModel?, wanted: (DocumentModel) -> Boolean): Int {
        val pending = dashboard?.tasks.orEmpty()
            .filterIsInstance<OpenTask.ReviewSuggestion>()
            .map { it.suggestion.documentId }
            .toSet()
        return services.library.catalog.value?.documents?.values.orEmpty()
            .count { it.kind.readableByAi && it.id !in pending && wanted(it) }
    }

    private fun nameOf(documentId: DocumentIdModel): String {
        return services.library.catalog.value?.documents?.get(documentId)?.path?.value ?: documentId.value
    }

    private fun toUiState(
        dashboard: DashboardModel?,
        sessionState: SessionStateModel,
        filter: TaskGroup?,
    ): DashboardUiStateModel {
        val context = TaskContext(folders = sessionState.folders, catalog = services.library.catalog.value)
        val byGroup = dashboard?.tasks.orEmpty().groupBy { TaskGroups.of(task = it, context = context) }
        val foreign = (sessionState.status as? LibraryStatus.Open)?.foreignFolders.orEmpty()
        return DashboardUiStateModel(
            folders = dashboard?.folders.orEmpty().map { toFolderStatus(state = it, foreign = foreign) },
            primaryFolderId = sessionState.folders?.primary?.id,
            documentCount = dashboard?.documentCount ?: 0,
            uncategorizedCount = dashboard?.uncategorizedCount ?: 0,
            groups = TaskGroup.entries.mapNotNull { group ->
                byGroup[group]?.let { tasks -> toGroupState(group = group, tasks = tasks, context = context) }
            },
            filter = filter,
            activity = dashboard?.recentActivity.orEmpty(),
            lastScanAt = sessionState.lastScanAt,
            busy = sessionState.busy,
            aiBusy = sessionState.aiBusy,
            classifyCandidates = aiCandidates(dashboard = dashboard) { it.classification == null },
            tagCandidates = aiCandidates(dashboard = dashboard) { it.tags.isEmpty() },
            inbox = inboxOf(catalog = services.library.catalog.value),
            upcoming = upcomingOf(dashboard = dashboard, catalog = services.library.catalog.value),
            stats = LibraryStatistics.of(
                catalog = services.library.catalog.value,
                now = services.library.clock.now(),
                zone = ZoneId.systemDefault(),
            ),
        )
    }

    private fun inboxOf(catalog: CatalogModel?): InboxSummaryModel? {
        val inbox = catalog?.settings?.layout?.inbox?.takeIf { it.enabled } ?: return null
        return InboxSummaryModel(
            path = inbox.path.value,
            count = catalog.documents.values.count { inbox.contains(candidate = it.path) },
            aiAnalysis = inbox.aiAnalysis,
        )
    }

    private fun upcomingOf(dashboard: DashboardModel?, catalog: CatalogModel?): List<UpcomingRowModel> {
        val context = TaskContext(folders = null, catalog = catalog)
        return dashboard?.upcoming.orEmpty().map { upcoming ->
            val document = catalog?.documents?.get(upcoming.documentId)
            UpcomingRowModel(
                documentId = upcoming.documentId,
                date = upcoming.date.toString(),
                what = upcoming.field?.let { context.fieldLabel(key = it) } ?: "Retention ends",
                document = document?.metadata?.title ?: document?.path?.fileName ?: upcoming.documentId.value,
                path = document?.path,
            )
        }
    }

    private fun toGroupState(group: TaskGroup, tasks: List<OpenTask>, context: TaskContext): TaskGroupStateModel {
        val rows = tasks.map { task ->
            val files = TaskActions.filesFor(task = task, context = context)
            // The copies of a duplicates task carry their own Open buttons.
            val copies = TaskActions.copiesFor(task = task, context = context, handler = this)
            TaskRowStateModel(
                key = task.key,
                title = TaskTexts.title(task = task, context = context),
                detail = TaskTexts.detail(task = task, context = context),
                buttons = TaskActions.buttonsFor(task = task, context = context, handler = this, organize = organize),
                opens = if (copies.isEmpty()) TaskActions.opensOf(files = files, handler = this) else emptyList(),
                preview = files.firstOrNull(),
                copies = copies,
            )
        }
        return TaskGroupStateModel(
            group = group,
            rows = rows,
            bulkActions = bulk.buttonsFor(group = group, tasks = tasks, context = context) +
                organize.bulkButtonsFor(group = group, tasks = tasks),
        )
    }

    private fun toFolderStatus(state: FolderStateModel, foreign: Set<LibraryFolderIdModel>): FolderStatusModel {
        val health = when {
            state.folder.id in foreign -> FolderHealth.FOREIGN
            state.reachable == null -> FolderHealth.NOT_SCANNED
            state.reachable == true -> FolderHealth.REACHABLE
            else -> FolderHealth.OFFLINE
        }
        return FolderStatusModel(
            name = state.folder.displayName,
            location = state.folder.location.value,
            isPrimary = state.folder.role == FolderRole.PRIMARY,
            health = health,
            pendingOperations = state.pendingOperations,
        )
    }
}
