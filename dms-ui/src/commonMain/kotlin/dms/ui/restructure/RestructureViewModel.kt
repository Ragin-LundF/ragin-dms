package dms.ui.restructure

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dms.application.restructure.model.PlanOutcome
import dms.application.restructure.model.RestructureOutcome
import dms.application.restructure.model.RestructureScope
import dms.domain.library.model.RelativePathModel
import dms.domain.restructure.FileNaming
import dms.domain.restructure.SkipReason
import dms.domain.restructure.model.RestructurePlanModel
import dms.domain.restructure.template.TemplateProblem
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ConfirmationRequestModel
import dms.ui.restructure.model.MoveGroupModel
import dms.ui.restructure.model.RestructureUiStateModel
import dms.ui.restructure.model.ScopeOptionModel
import dms.ui.review.ReviewDrafts
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.OperationProgressModel
import dms.ui.session.model.UiMessageModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * The restructure assistant: the user picks a scope and a path template, sees where every document
 * would go, unticks what should stay, and applies the moves. Planning only reads; applying runs as one
 * cancellable library operation, and every applied restructure can be reversed.
 */
class RestructureViewModel(
    private val services: AppServices,
    private val session: LibrarySession,
) : ViewModel() {

    private val mutableState = MutableStateFlow(value = RestructureUiStateModel())
    private var plan: RestructurePlanModel? = null

    val state: StateFlow<RestructureUiStateModel> = mutableState.asStateFlow()
    val dialogs: DialogController = DialogController()

    init {
        refresh()
    }

    /** Reloads the scopes, the saved template and the history, keeping what the user typed. */
    fun refresh() {
        val catalog = services.library.catalog.value
        val folders = catalog?.let { ReviewDrafts.folderPathsOf(catalog = it) }.orEmpty()
        val everything = ScopeOptionModel(
            label = "Whole library",
            scope = RestructureScope.Folder(path = RelativePathModel.ROOT),
        )
        val scopes = listOf(everything) +
            catalog?.settings?.savedSearches.orEmpty().map {
                ScopeOptionModel(label = "Smart folder: ${it.name}", scope = RestructureScope.Query(query = it.query))
            } +
            folders.map { folder ->
                val scope = RestructureScope.Folder(path = RelativePathModel.of(raw = folder))
                ScopeOptionModel(label = folder, scope = scope)
            }
        val saved = catalog?.settings?.layout?.pathTemplate
        mutableState.update { current ->
            current.copy(
                scopes = scopes,
                scopeIndex = current.scopeIndex.coerceIn(range = 0..scopes.lastIndex),
                template = current.template.ifEmpty { saved ?: DEFAULT_TEMPLATE },
                savedTemplate = saved,
                busy = session.state.value.busy,
            )
        }
        viewModelScope.launch {
            val history = services.organize.reverseRestructure.history()
            mutableState.update { it.copy(history = history) }
        }
    }

    /** Opens the assistant on [folder], as a flat-folder task suggests it. */
    fun start(folder: RelativePathModel) {
        refresh()
        val index = state.value.scopes.indexOfFirst { (it.scope as? RestructureScope.Folder)?.path == folder }
        if (index >= 0) {
            selectScope(index = index)
            preview()
        }
    }

    fun selectScope(index: Int) {
        mutableState.update { it.copy(scopeIndex = index, planned = false, groups = emptyList()) }
    }

    fun setTemplate(text: String) {
        mutableState.update { it.copy(template = text, problem = null, planned = false) }
    }

    fun insertPlaceholder(token: String) {
        mutableState.update { current ->
            val separator = if (current.template.isEmpty() || current.template.endsWith(suffix = "/")) "" else "/"
            current.copy(template = current.template + separator + "{$token}", planned = false)
        }
    }

    fun setNaming(naming: FileNaming) {
        mutableState.update { it.copy(naming = naming, planned = false) }
    }

    /** Plans the restructure for the chosen scope and template; nothing changes. */
    fun preview() {
        val current = state.value
        val scope = current.scopes.getOrNull(index = current.scopeIndex)?.scope ?: return
        viewModelScope.launch {
            when (val outcome = services.organize.planRestructure(
                scope = scope,
                template = current.template,
                naming = current.naming,
            )) {
                is PlanOutcome.Planned -> show(plan = outcome.plan)
                is PlanOutcome.InvalidTemplate -> mutableState.update {
                    it.copy(problem = problemText(problem = outcome.problem))
                }

                PlanOutcome.LibraryNotOpen -> mutableState.update { it.copy(problem = "No library is open") }
            }
        }
    }

    fun toggleGroup(folder: String) {
        mutableState.update { current ->
            current.copy(
                groups = current.groups.map { if (it.folder == folder) it.copy(included = !it.included) else it },
            )
        }
    }

    /** Remembers the template as the library's default; inbox filing pre-fills its folder from it. */
    fun saveTemplate() {
        val template = state.value.template.trim()
        session.perform(label = "Saving the path template") {
            val outcome = services.documents.changeSettings(subject = "Path template $template") { settings ->
                settings.withLayout(layout = settings.layout.copy(pathTemplate = template))
            }
            session.report(
                message = OutcomeMessages.of(outcome = outcome, done = "Saved $template as the default template"),
            )
        }
        mutableState.update { it.copy(savedTemplate = template) }
    }

    fun apply() {
        val planned = plan ?: return
        val included = state.value.groups.filter { it.included }.map { it.folder }.toSet()
        val chosen = planned.only { (it.to.parent ?: RelativePathModel.ROOT).value in included }
        if (chosen.moves.isEmpty()) {
            return
        }
        dialogs.ask(
            request = ConfirmationRequestModel(
                title = "Move ${chosen.moves.size} documents",
                message = "The files move in every library folder. You can reverse this restructure later.",
                affected = affectedOf(plan = chosen),
                confirmLabel = "Move",
            ),
        ) {
            runRestructure(label = "Restructuring ${chosen.moves.size} documents") { onProgress ->
                services.organize.applyRestructurePlan(plan = chosen, onProgress = onProgress)
            }
        }
    }

    private fun affectedOf(plan: RestructurePlanModel): List<String> {
        val shown = plan.moves.take(n = AFFECTED_SHOWN).map { "${it.from} → ${it.to}" }
        val more = plan.moves.size - AFFECTED_SHOWN
        return if (more > 0) shown + "… and $more more" else shown
    }

    fun reverse(name: String) {
        dialogs.ask(
            request = ConfirmationRequestModel(
                title = "Reverse restructure $name",
                message = "Every document still where this restructure put it goes back. Documents moved since stay.",
                affected = listOf(name),
                confirmLabel = "Reverse",
            ),
        ) {
            runRestructure(label = "Reversing restructure $name") { onProgress ->
                services.organize.reverseRestructure(name = name, onProgress = onProgress)
            }
        }
    }

    private fun runRestructure(label: String, action: suspend (onProgress: (Int, Int) -> Unit) -> RestructureOutcome) {
        session.perform(label = label, cancellable = true) {
            val outcome = action { done, total ->
                session.progress(progress = OperationProgressModel(step = label, done = done, total = total))
            }
            report(outcome = outcome)
            refresh()
            mutableState.update { it.copy(planned = false, groups = emptyList()) }
        }
    }

    private fun report(outcome: RestructureOutcome) {
        when (outcome) {
            RestructureOutcome.LibraryNotOpen -> session.report(
                message = UiMessageModel(text = "No library is open", isError = true),
            )
            RestructureOutcome.RecordNotFound -> session.report(
                message = UiMessageModel(text = "The record of this restructure cannot be read", isError = true),
            )

            is RestructureOutcome.Applied -> {
                val parts = listOfNotNull(
                    "Moved ${outcome.moved} documents",
                    "${outcome.failed.size} could not be moved".takeIf { outcome.failed.isNotEmpty() },
                    "${outcome.skipped} changed since and stayed".takeIf { outcome.skipped > 0 },
                    "${outcome.pendingFolders.size} folders wait in the journal"
                        .takeIf { outcome.pendingFolders.isNotEmpty() },
                )
                session.report(
                    message = UiMessageModel(
                        text = parts.joinToString(separator = "; "),
                        isError = outcome.failed.isNotEmpty(),
                    ),
                )
                outcome.failed.forEach { session.log.error(text = "Not moved: ${it.from}") }
                offerRemoval(folders = outcome.emptyFolders)
            }
        }
    }

    private fun offerRemoval(folders: List<RelativePathModel>) {
        if (folders.isEmpty()) {
            return
        }
        dialogs.ask(
            request = ConfirmationRequestModel(
                title = "Remove empty folders",
                message = "These folders hold no documents any more. They go to the trash in every library folder.",
                affected = folders.map { it.value },
                confirmLabel = "Move to trash",
            ),
        ) {
            session.perform(label = "Removing ${folders.size} empty folders") {
                val outcome = services.organize.removeEmptyFolders(folders = folders, confirmed = true)
                session.report(message = OutcomeMessages.of(outcome = outcome, done = "Removed empty folders"))
            }
        }
    }

    private fun show(plan: RestructurePlanModel) {
        this.plan = plan
        val names = services.library.catalog.value?.documents.orEmpty()
        val groups = plan.moves.groupBy { (it.to.parent ?: RelativePathModel.ROOT).value }.toSortedMap().map { (
            folder,
            moves,
        ) ->
            MoveGroupModel(
                folder = folder,
                count = moves.size,
                samples = moves.take(n = SAMPLES).map { "${it.from} → ${it.to.fileName}" },
                included = true,
            )
        }
        mutableState.update { current ->
            current.copy(
                planned = true,
                problem = null,
                groups = groups,
                unchanged = plan.unchanged.size,
                skipped = plan.skipped.groupingBy { it.reason }.eachCount()
                    .map { (reason, count) -> "$count ${skipText(reason = reason)}" },
                renamed = plan.moves.filter { it.documentId in plan.renamedForClash }
                    .map { "${names[it.documentId]?.path?.fileName} → ${it.to}" },
            )
        }
    }

    private fun skipText(reason: SkipReason): String {
        return when (reason) {
            SkipReason.MISSING_CATEGORY -> "without category"
            SkipReason.MISSING_DATE -> "without date"
            SkipReason.MISSING_CORRESPONDENT -> "without correspondent"
            SkipReason.MISSING_FIELD -> "without the field value"
            SkipReason.IN_INBOX -> "in the inbox"
        }
    }

    private fun problemText(problem: TemplateProblem): String {
        return when (problem) {
            TemplateProblem.EMPTY -> "Write a template, such as {category}/{year}"
            TemplateProblem.UNCLOSED_BRACE -> "A { has no closing }"
            TemplateProblem.UNKNOWN_PLACEHOLDER -> "Unknown placeholder; use the chips below"
            TemplateProblem.INVALID_FIELD_KEY -> "Write fields as {field:key}, such as {field:due}"
            TemplateProblem.INVALID_FOLDER_NAME -> "A folder name is not allowed or contains one of < > : \" \\ | ? *"
        }
    }

    private companion object {
        const val DEFAULT_TEMPLATE = "{category}/{year|Undated}"
        const val SAMPLES = 3
        const val AFFECTED_SHOWN = 12
    }
}
