package dms.ui.library.smart

import dms.application.search.SavedSearchPresets
import dms.application.search.model.FindOutcome
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.QueryContextModel
import dms.domain.query.model.SavedSearchIdModel
import dms.domain.query.model.SavedSearchModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ConfirmationRequestModel
import dms.ui.components.model.TextInputRequestModel
import dms.ui.library.smart.model.SmartFolderEntryModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.launch
import java.time.ZoneId
import java.util.UUID

/**
 * Smart folders: saved searches shown in the library sidebar, synced with the catalog. The library's
 * view model owns it and hands it what the user filters now ([currentQuery]); a smart folder whose
 * query needs the documents' text gets its matches from the content search in the background.
 */
class SmartFolders(
    private val services: AppServices,
    private val session: LibrarySession,
    private val dialogs: DialogController,
    private val currentQuery: () -> DocumentQueryModel,
    selected: Flow<SavedSearchIdModel?>,
    scope: CoroutineScope,
) {

    private val matchesState = MutableStateFlow<Set<DocumentIdModel>?>(value = null)

    /** The documents the selected smart folder finds by text; `null` when it needs no text. */
    val textMatches: StateFlow<Set<DocumentIdModel>?> = matchesState.asStateFlow()

    init {
        scope.launch {
            combine(flow = selected, flow2 = services.library.catalog) { id, catalog ->
                catalog?.settings?.savedSearches?.firstOrNull { it.id == id }?.query
            }.collectLatest { query ->
                matchesState.value = if (query == null || !query.needsContent) {
                    null
                } else {
                    when (val found = services.library.findDocuments(query = query)) {
                        is FindOutcome.Found -> found.documents.map { it.id }.toSet()
                        FindOutcome.LibraryNotOpen -> emptySet()
                    }
                }
            }
        }
    }

    /** The smart folders of [catalog] with their counts, in the order the user created them. */
    fun entriesOf(catalog: CatalogModel): List<SmartFolderEntryModel> {
        val clock = services.library.clock
        val context = QueryContextModel(catalog = catalog, today = clock.today(), zone = ZoneId.systemDefault())
        return catalog.settings.savedSearches.map { search ->
            SmartFolderEntryModel(
                id = search.id,
                name = search.name,
                count = if (search.query.needsContent) null else search.query.filter(context = context).size,
                invalid = search.query.refersToMissing(catalog = catalog),
            )
        }
    }

    /** Asks for a name and saves what the library shows now as a smart folder. */
    fun saveCurrent() {
        val query = currentQuery()
        ask(title = "Save as smart folder", initial = "") { name ->
            save(
                search = SavedSearchModel(
                    id = SavedSearchIdModel(value = UUID.randomUUID().toString()),
                    name = name,
                    query = query,
                ),
            )
        }
    }

    /** Replaces the query of a smart folder with what the library shows now. */
    fun replaceWithCurrent(id: SavedSearchIdModel) {
        val search = searchOf(id = id) ?: return
        save(search = search.copy(query = currentQuery()))
    }

    fun rename(id: SavedSearchIdModel) {
        val search = searchOf(id = id) ?: return
        ask(title = "Rename smart folder", initial = search.name) { name -> save(search = search.copy(name = name)) }
    }

    fun delete(id: SavedSearchIdModel) {
        val search = searchOf(id = id) ?: return
        dialogs.ask(
            request = ConfirmationRequestModel(
                title = "Delete smart folder \"${search.name}\"",
                message = "Only the saved search is deleted; no document changes.",
                affected = emptyList(),
                confirmLabel = "Delete",
            ),
        ) {
            change(done = "Deleted ${search.name}") { it.withoutSearch(id = id) }
        }
    }

    /** Adds "Untagged", "Uncategorized", "Tax <year>" and "Due in 30 days" where no folder has the name. */
    fun addPresets() {
        val catalog = services.library.catalog.value ?: return
        val taxTag = catalog.tags.firstOrNull { it == TagNameModel.of(raw = "Tax") } ?: TagNameModel.of(raw = "Tax")
        val presets = SavedSearchPresets.of(
            today = services.library.clock.today(),
            ids = { UUID.randomUUID().toString() },
            taxTag = taxTag,
        )
        change(done = "Added the standard smart folders") { settings ->
            val taken = settings.savedSearches.map { it.name.lowercase() }.toSet()
            presets.filterNot { it.name.lowercase() in taken }
                .fold(initial = settings) { current, preset -> current.withSearch(search = preset) }
        }
    }

    private fun save(search: SavedSearchModel) {
        change(done = "Saved ${search.name}") { it.withSearch(search = search) }
    }

    private fun change(done: String, change: (CatalogSettingsModel) -> CatalogSettingsModel) {
        session.perform(label = "Saving the smart folders") {
            val outcome = services.documents.changeSettings(subject = "Smart folders", change = change)
            session.report(message = OutcomeMessages.of(outcome = outcome, done = done))
        }
    }

    private fun ask(title: String, initial: String, submit: (String) -> Unit) {
        dialogs.askText(
            request = TextInputRequestModel(
                title = title,
                label = "Name",
                initialValue = initial,
                confirmLabel = "Save",
            ),
        ) { name ->
            name.trim().takeIf { it.isNotEmpty() }?.let(submit)
        }
    }

    fun searchOf(id: SavedSearchIdModel): SavedSearchModel? {
        return services.library.catalog.value?.settings?.savedSearches?.firstOrNull { it.id == id }
    }
}
