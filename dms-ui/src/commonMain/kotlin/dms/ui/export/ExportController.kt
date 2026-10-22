package dms.ui.export

import dms.application.export.model.ExportOptionsModel
import dms.application.export.model.ExportOutcome
import dms.application.export.model.ExportScope
import dms.domain.query.model.DateRangeModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.SavedSearchModel
import dms.domain.query.model.TagFilterModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.PlatformActions
import dms.ui.export.model.ExportDraftModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.UiMessageModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * Exports documents into a folder or a ZIP file outside the library: from the current view, a smart
 * folder, or a preset such as a tax year or everything of one correspondent. The user picks the
 * target folder; the export only reads the library.
 */
class ExportController(
    private val services: AppServices,
    private val session: LibrarySession,
    private val platform: PlatformActions,
    private val scope: CoroutineScope,
    private val currentQuery: () -> DocumentQueryModel,
) {

    private val draftState = MutableStateFlow<ExportDraftModel?>(value = null)

    val draft: StateFlow<ExportDraftModel?> = draftState.asStateFlow()

    /** The tax years offered as presets: this year and the three before. */
    val taxYears: List<Int>
        get() {
            val year = services.library.clock.today().year
            return (year downTo year - TAX_YEARS + 1).toList()
        }

    fun openCurrentView(title: String) {
        draftState.value = ExportDraftModel(
            scope = ExportScope.Query(query = currentQuery()),
            scopeLabel = title,
            name = title,
        )
    }

    fun openSmartFolder(search: SavedSearchModel) {
        draftState.value = ExportDraftModel(
            scope = ExportScope.Query(query = search.query),
            scopeLabel = search.name,
            name = search.name,
        )
    }

    /** The documents tagged with the tax tag and dated in [year]. */
    fun presetTaxYear(year: Int) {
        val wanted = TagNameModel.of(raw = "Tax")
        val tax = services.library.catalog.value?.tags?.firstOrNull { it == wanted } ?: wanted
        val query = DocumentQueryModel(
            tags = TagFilterModel(all = setOf(tax)),
            dates = DateRangeModel.year(year = year),
        )
        draftState.update {
            it?.copy(scope = ExportScope.Query(query = query), scopeLabel = "Tax year $year", name = "Tax $year")
        }
    }

    fun setCorrespondent(name: String) {
        draftState.update { it?.copy(correspondent = name) }
    }

    /** Everything from the correspondent typed. */
    fun presetCorrespondent() {
        val name = draftState.value?.correspondent?.trim()?.takeIf { it.isNotEmpty() } ?: return
        draftState.update {
            it?.copy(
                scope = ExportScope.Query(query = DocumentQueryModel(correspondent = name)),
                scopeLabel = "Everything from $name",
                name = name,
            )
        }
    }

    fun setName(name: String) {
        draftState.update { it?.copy(name = name) }
    }

    fun setOptions(options: ExportOptionsModel) {
        draftState.update { it?.copy(options = options) }
    }

    fun cancel() {
        draftState.value = null
    }

    /** Asks for the target folder and exports there. */
    fun export() {
        val chosen = draftState.value ?: return
        if (chosen.name.isBlank()) {
            return
        }
        scope.launch {
            val directory = platform.chooseFolder(title = "Choose where to export \"${chosen.name}\"") ?: return@launch
            val started = session.perform(label = "Exporting ${chosen.scopeLabel}") {
                val outcome = services.organize.exportDocuments(
                    scope = chosen.scope,
                    directory = directory,
                    name = chosen.name.trim(),
                    options = chosen.options,
                )
                session.report(message = messageOf(outcome = outcome))
            }
            if (started) {
                draftState.value = null
            }
        }
    }

    private fun messageOf(outcome: ExportOutcome): UiMessageModel {
        return when (outcome) {
            is ExportOutcome.Exported -> UiMessageModel(
                text = "Exported ${outcome.count} documents to ${outcome.location}",
            )
            ExportOutcome.Empty -> UiMessageModel(text = "No document matches; nothing was exported", isError = true)
            ExportOutcome.InsideLibrary -> UiMessageModel(text = "Choose a folder outside the library", isError = true)
            ExportOutcome.LibraryNotOpen -> UiMessageModel(text = "No library is open", isError = true)
            is ExportOutcome.Failed -> UiMessageModel(
                text = "The export failed: ${OutcomeMessages.failure(reason = outcome.failure.reason)}",
                isError = true,
            )
        }
    }

    private companion object {
        const val TAX_YEARS = 4
    }
}
