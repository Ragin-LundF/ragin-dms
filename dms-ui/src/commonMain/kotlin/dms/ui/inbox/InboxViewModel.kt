package dms.ui.inbox

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dms.application.common.OperationOutcome
import dms.domain.catalog.model.CatalogModel
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.restructure.template.TemplateParse
import dms.domain.restructure.template.TemplateRendering
import dms.domain.restructure.template.model.PathTemplateModel
import dms.domain.task.OpenTask
import dms.ui.components.FileSizes
import dms.ui.dashboard.ActivityTexts
import dms.ui.inbox.model.FolderPickModel
import dms.ui.inbox.model.InboxRowModel
import dms.ui.inbox.model.InboxUiStateModel
import dms.ui.review.AiMessages
import dms.ui.review.ReviewController
import dms.ui.review.ReviewDrafts
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.OperationLane
import dms.ui.session.model.SessionStateModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

/**
 * The inbox: documents dropped into the inbox folder of any library folder. Each one is analyzed by
 * the AI (a suggestion to review, optionally with a target folder) or moved by hand to a folder the
 * user picks, without any AI.
 */
class InboxViewModel(
    private val services: AppServices,
    private val session: LibrarySession,
) : ViewModel() {

    private val suggestions = MutableStateFlow<Map<DocumentIdModel, ClassificationSuggestionModel>>(value = emptyMap())
    private val pick = MutableStateFlow<FolderPickModel?>(value = null)
    private val recent = MutableStateFlow<List<String>>(value = emptyList())

    val review: ReviewController = ReviewController(services = services, session = session)

    val state: StateFlow<InboxUiStateModel> = combine(
        flow = services.library.catalog,
        flow2 = session.state,
        flow3 = suggestions,
        flow4 = pick,
        flow5 = recent,
    ) { catalog, sessionState, waiting, picking, recentTargets ->
        toUiState(
            catalog = catalog,
            sessionState = sessionState,
            waiting = waiting,
            pick = picking,
            recent = recentTargets,
        )
    }.stateIn(scope = viewModelScope, started = SharingStarted.Eagerly, initialValue = InboxUiStateModel())

    init {
        viewModelScope.launch {
            session.state.map { it.revision to it.lastScan }.distinctUntilChanged().collectLatest { (_, scan) ->
                val tasks = services.library.buildDashboard(scan = scan)?.tasks.orEmpty()
                suggestions.value = tasks.filterIsInstance<OpenTask.ReviewSuggestion>()
                    .associate { it.suggestion.documentId to it.suggestion }
            }
        }
    }

    fun analyze(documentId: DocumentIdModel) {
        val name = nameOf(documentId = documentId)
        session.perform(label = "Analyzing $name", lane = OperationLane.AI) {
            val outcome = services.ai.suggestClassification(documentId = documentId)
            session.report(message = AiMessages.of(outcome = outcome, name = name))
            session.changed()
        }
    }

    fun analyzeAll() {
        session.analyzeInbox()
    }

    fun review(suggestion: ClassificationSuggestionModel) {
        review.open(suggestion = suggestion)
    }

    /** Opens "Move to…" for [documentId], with the folder the saved path template gives it, if any. */
    fun startMove(documentId: DocumentIdModel) {
        val row = state.value.rows.firstOrNull { it.documentId == documentId } ?: return
        pick.value = FolderPickModel(
            documentId = documentId,
            documentName = row.name,
            folder = row.prefill ?: recent.value.firstOrNull().orEmpty(),
            fileName = row.name,
        )
    }

    fun updatePick(folder: String, fileName: String) {
        pick.value = pick.value?.copy(folder = folder, fileName = fileName)
    }

    fun cancelMove() {
        pick.value = null
    }

    /** Files the picked document into the picked folder; a new folder is created on the way. */
    fun confirmMove() {
        val chosen = pick.value ?: return
        val folder = RelativePathModel.fromSegments(
            segments = chosen.folder.split('/').map { it.trim() }.filter { it.isNotEmpty() },
        )
        val fileName = chosen.fileName.trim().takeIf { it.isNotEmpty() && it != chosen.documentName }
        val started = session.perform(label = "Filing ${chosen.documentName}") {
            val outcome = services.organize.fileInboxDocument(
                documentId = chosen.documentId,
                targetFolder = folder,
                newName = fileName,
            )
            session.report(
                message = OutcomeMessages.of(outcome = outcome, done = "Filed ${chosen.documentName} in $folder"),
            )
            if (outcome is OperationOutcome.Done) {
                recent.value = (listOf(folder.value) + recent.value).distinct().take(n = RECENT_TARGETS)
            }
        }
        if (started) {
            pick.value = null
        }
    }

    private fun nameOf(documentId: DocumentIdModel): String {
        return services.library.catalog.value?.documents?.get(documentId)?.path?.fileName ?: documentId.value
    }

    private fun toUiState(
        catalog: CatalogModel?,
        sessionState: SessionStateModel,
        waiting: Map<DocumentIdModel, ClassificationSuggestionModel>,
        pick: FolderPickModel?,
        recent: List<String>,
    ): InboxUiStateModel {
        val inbox = catalog?.settings?.layout?.inbox ?: return InboxUiStateModel(busy = sessionState.busy)
        val template = catalog.settings.layout.pathTemplate
            ?.let { (PathTemplateModel.parse(raw = it) as? TemplateParse.Parsed)?.template }
        val rows = catalog.documents.values
            .filter { inbox.contains(candidate = it.path) }
            .sortedByDescending { it.addedAt }
            .map { document ->
                val rendered = template?.render(document = document, catalog = catalog) as? TemplateRendering.Rendered
                toRow(
                    document = document,
                    inboxPath = inbox.path,
                    suggestion = waiting[document.id],
                    prefill = rendered?.folder?.value,
                )
            }
        return InboxUiStateModel(
            enabled = inbox.enabled,
            path = inbox.path.value,
            aiAnalysis = inbox.aiAnalysis,
            rows = rows,
            folders = ReviewDrafts.folderPathsOf(catalog = catalog)
                .filterNot { it == inbox.path.value || it.startsWith(prefix = inbox.path.value + "/") },
            recentTargets = recent,
            pick = pick,
            busy = sessionState.busy,
            aiBusy = sessionState.aiBusy,
        )
    }

    private fun toRow(
        document: DocumentModel,
        inboxPath: RelativePathModel,
        suggestion: ClassificationSuggestionModel?,
        prefill: String?,
    ): InboxRowModel {
        val parent = document.path.parent ?: RelativePathModel.ROOT
        return InboxRowModel(
            documentId = document.id,
            name = document.path.fileName,
            subfolder = if (parent == inboxPath) "" else parent.value.removePrefix(prefix = inboxPath.value + "/"),
            added = ActivityTexts.time(instant = document.addedAt),
            size = FileSizes.format(bytes = document.size),
            suggestion = suggestion,
            analyzable = document.kind.readableByAi,
            prefill = prefill?.takeIf { it.isNotEmpty() },
        )
    }

    private companion object {
        const val RECENT_TARGETS = 5
    }
}
