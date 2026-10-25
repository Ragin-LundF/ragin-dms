package dms.ui.dashboard.organize

import dms.application.common.OperationOutcome
import dms.application.deadline.DeadlineCalendar
import dms.application.pdf.model.ScansOutcome
import dms.application.pdf.model.SearchableOutcome
import dms.application.port.storage.model.StorageResult
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.rule.learn.model.RuleProposalModel
import dms.domain.task.OpenTask
import dms.ui.PlatformActions
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ConfirmationRequestModel
import dms.ui.dashboard.model.TaskGroup
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.OperationProgressModel
import dms.ui.session.model.UiMessageModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * Answers the organizing tasks of the dashboard. Opening another screen is a request the app's
 * content observes ([restructureRequests]), so the dashboard never navigates by itself.
 */
class OrganizeTasks(
    private val services: AppServices,
    private val session: LibrarySession,
    private val dialogs: DialogController,
    private val platform: PlatformActions,
    private val scope: CoroutineScope,
) : OrganizeTaskHandler {

    private val restructureState = MutableStateFlow<RelativePathModel?>(value = null)
    private val scansState = MutableStateFlow<List<DocumentIdModel>?>(value = null)

    /** The PDFs found without a text layer, once the user looked for them. */
    val scans: StateFlow<List<DocumentIdModel>?> = scansState.asStateFlow()

    /** The folder the user wants to restructure, until the restructure screen took it. */
    val restructureRequests: StateFlow<RelativePathModel?> = restructureState.asStateFlow()

    override fun restructure(folder: RelativePathModel) {
        restructureState.value = folder
    }

    fun consumeRestructureRequest() {
        restructureState.value = null
    }

    override fun trashRetained(documentIds: List<DocumentIdModel>) {
        val documents = services.library.catalog.value?.documents.orEmpty()
        val paths = documentIds.mapNotNull { documents[it]?.path?.value }
        val folders = session.state.value.folders?.folders.orEmpty()
        dialogs.ask(
            request = ConfirmationRequestModel(
                title = "Move ${paths.size} documents to the trash",
                message = "You chose to keep them until a date that has passed. They go to the trash in every " +
                    "library folder and leave the catalog.",
                affected = paths.flatMap { path -> folders.map { "${it.displayName}: $path" } },
                confirmLabel = "Move to trash",
            ),
        ) {
            session.perform(label = "Moving ${paths.size} documents to the trash", cancellable = true) {
                val trashed = session.forEachWithProgress(
                    step = "Moving to trash",
                    items = documentIds,
                    name = { id -> documents[id]?.path?.value ?: id.value },
                ) { id ->
                    services.documents.trashDocument(documentId = id, confirmed = true) is OperationOutcome.Done
                }
                session.report(
                    message = UiMessageModel(text = "Moved $trashed of ${documentIds.size} documents to the trash"),
                )
            }
        }
    }

    override fun settleRetention(documentIds: List<DocumentIdModel>) {
        session.perform(label = "Marking the retention as done") {
            val outcome = services.documents.settleRetention(documentIds = documentIds)
            val done = "Kept ${documentIds.size} documents; their retention is done"
            session.report(message = OutcomeMessages.of(outcome = outcome, done = done))
        }
    }

    override fun acceptRule(proposal: RuleProposalModel) {
        session.perform(label = "Adding the rule for ${proposal.correspondent}") {
            val outcome = services.organize.acceptRuleProposal(proposal = proposal)
            val done = "Documents of ${proposal.correspondent} now get the rule's values proposed"
            session.report(message = OutcomeMessages.of(outcome = outcome, done = done))
        }
    }

    /** Asks for a folder and writes the upcoming deadlines and retention dates there as a calendar file. */
    fun exportCalendar() {
        val catalog = services.library.catalog.value ?: return
        val calendar = DeadlineCalendar.of(catalog = catalog, today = services.library.clock.today())
        scope.launch {
            val directory = platform.chooseFolder(title = "Choose where to save the calendar") ?: return@launch
            session.perform(label = "Saving the calendar") {
                val written = services.organize.exportWriter.writeText(
                    directory = directory,
                    name = "deadlines.ics",
                    text = calendar,
                )
                val message = when (written) {
                    is StorageResult.Success -> UiMessageModel(text = "Saved ${written.value}")
                    is StorageResult.Failure -> UiMessageModel(
                        text = "The calendar could not be saved: ${OutcomeMessages.failure(reason = written.reason)}",
                        isError = true,
                    )
                }
                session.report(message = message)
            }
        }
    }

    /** Looks for PDFs without a text layer in the background; it only reads. */
    fun findScans() {
        scope.launch { scansState.value = scansWithoutText() }
    }

    /** Makes every scan without text searchable from its transcription, one after the other; can be cancelled. */
    fun makeScansSearchable() {
        session.perform(label = "Making scans searchable", cancellable = true) {
            var made = 0
            services.pdf.makeAllSearchable().collect { progress ->
                val step = OperationProgressModel(
                    step = "Making scans searchable",
                    done = progress.done,
                    total = progress.total,
                )
                session.progress(progress = step)
                if (progress.outcome is SearchableOutcome.Made) {
                    made++
                }
            }
            session.report(
                message = UiMessageModel(
                    text = "Made $made scans searchable; scans without a transcription were left out",
                ),
            )
            scansState.value = scansWithoutText()
        }
    }

    private suspend fun scansWithoutText(): List<DocumentIdModel> {
        return when (val outcome = services.pdf.scansWithoutText()) {
            is ScansOutcome.Found -> outcome.documentIds
            ScansOutcome.LibraryNotOpen -> emptyList()
        }
    }

    /** The buttons that answer every retention task of the group at once. */
    fun bulkButtonsFor(group: TaskGroup, tasks: List<OpenTask>): List<ActionButtonModel> {
        if (group != TaskGroup.RETENTION) {
            return emptyList()
        }
        val ids = tasks.filterIsInstance<OpenTask.RetentionDue>().map { it.documentId }
        return listOf(
            ActionButtonModel(label = "Mark all as done (${ids.size})") { settleRetention(documentIds = ids) },
            ActionButtonModel(label = "Move all to trash (${ids.size})…") { trashRetained(documentIds = ids) },
        )
    }
}
