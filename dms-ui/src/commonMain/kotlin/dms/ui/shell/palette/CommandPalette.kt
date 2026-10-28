package dms.ui.shell.palette

import dms.application.search.model.SearchOutcome
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.model.OperationLane
import dms.ui.shell.palette.model.PaletteGroupModel
import dms.ui.shell.palette.model.PaletteItemModel
import dms.ui.shell.palette.model.PaletteStateModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn

/**
 * The command palette: what is typed, what it finds and which line is selected. It only reads, so it
 * never waits for a busy library; choosing a line is up to the caller, which runs the same functions as
 * the buttons do. Actions whose lane is busy say why and are not chosen.
 */
@OptIn(FlowPreview::class, ExperimentalCoroutinesApi::class)
class CommandPalette(
    private val services: AppServices,
    private val session: LibrarySession,
    scope: CoroutineScope,
) {

    private val open = MutableStateFlow(value = false)
    private val query = MutableStateFlow(value = "")
    private val selected = MutableStateFlow(value = 0)
    private var memo: Pair<Inputs, List<PaletteGroupModel>>? = null

    private val textMatches: StateFlow<Set<DocumentIdModel>> = query
        .debounce(timeoutMillis = TYPING_PAUSE_MILLIS)
        .mapLatest { typed -> textMatchesOf(query = typed) }
        .stateIn(scope = scope, started = SharingStarted.Eagerly, initialValue = emptySet())

    private val places: StateFlow<List<PaletteItemModel.GoTo>> = services.library.catalog
        .map { PaletteItems.placesOf(catalog = it) }
        .stateIn(scope = scope, started = SharingStarted.Eagerly, initialValue = emptyList())

    private val busy: StateFlow<Map<OperationLane, String>> = session.state
        .map { state ->
            OperationLane.entries.mapNotNull { lane -> state.busyIn(lane = lane)?.let { lane to it } }.toMap()
        }
        .distinctUntilChanged()
        .stateIn(scope = scope, started = SharingStarted.Eagerly, initialValue = emptyMap())

    private val inputs: StateFlow<Inputs> = combine(
        flow = query,
        flow2 = services.library.catalog,
        flow3 = textMatches,
        flow4 = places,
        flow5 = busy,
    ) { typed, catalog, matches, allPlaces, running ->
        Inputs(
            query = typed,
            catalog = catalog,
            textMatches = matches,
            places = allPlaces,
            busy = running,
            aiEnabled = session.state.value.aiEnabled,
        )
    }
        .stateIn(scope = scope, started = SharingStarted.Eagerly, initialValue = currentInputs())

    val state: StateFlow<PaletteStateModel> = combine(
        flow = open,
        flow2 = selected,
        flow3 = inputs,
    ) { isOpen, index, at ->
        val groups = groupsFor(inputs = at)
        val count = groups.sumOf { it.items.size }
        PaletteStateModel(
            open = isOpen,
            query = at.query,
            groups = groups,
            selected = if (count == 0) 0 else index.coerceIn(range = 0 until count),
        )
    }.stateIn(scope = scope, started = SharingStarted.Eagerly, initialValue = PaletteStateModel())

    /** Opens the palette with [initial] typed, as the toolbar field does. */
    fun open(initial: String = "") {
        query.value = initial
        selected.value = 0
        open.value = true
    }

    fun close() {
        open.value = false
    }

    fun type(text: String) {
        query.value = text
        selected.value = 0
    }

    /** Moves the selection by [steps] lines, wrapping around at both ends. */
    fun move(steps: Int) {
        val count = currentItems().size
        if (count == 0) {
            return
        }
        selected.value = Math.floorMod(selected.value.coerceIn(range = 0 until count) + steps, count)
    }

    /**
     * The selected line, closing the palette; `null` when nothing is listed or the selected action waits
     * for its lane, which then stays open so its reason stays visible.
     */
    fun choose(): PaletteItemModel? {
        val items = currentItems()
        val item = items.getOrNull(index = selected.value.coerceAtMost(maximumValue = items.lastIndex)) ?: return null
        if (item is PaletteItemModel.Run && item.disabledReason != null) {
            return null
        }
        close()
        return item
    }

    /** "Ask the library" for what is typed, closing the palette; `null` for an empty query. */
    fun ask(): PaletteItemModel? {
        if (!session.state.value.aiEnabled) return null
        val question = query.value.trim().takeIf { it.isNotEmpty() } ?: return null
        close()
        return PaletteItemModel.AskLibrary(question = question)
    }

    /**
     * The lines for what is typed right now. Read from the inputs directly, not from [state], so a key
     * pressed right after typing acts on what was typed.
     */
    private fun currentItems(): List<PaletteItemModel> {
        return groupsFor(inputs = currentInputs()).flatMap { it.items }
    }

    private fun currentInputs(): Inputs {
        return Inputs(
            query = query.value,
            catalog = services.library.catalog.value,
            textMatches = textMatches.value,
            places = places.value,
            busy = busy.value,
            aiEnabled = session.state.value.aiEnabled,
        )
    }

    /** The groups for [inputs], computed once for the same inputs. */
    @Synchronized
    private fun groupsFor(inputs: Inputs): List<PaletteGroupModel> {
        memo?.takeIf { it.first == inputs }?.let { return it.second }
        val groups = PaletteItems.of(
            query = inputs.query,
            catalog = inputs.catalog,
            textMatches = inputs.textMatches,
            places = inputs.places,
            busy = inputs.busy,
            aiEnabled = inputs.aiEnabled,
        )
        memo = inputs to groups
        return groups
    }

    private suspend fun textMatchesOf(query: String): Set<DocumentIdModel> {
        if (query.isBlank()) {
            return emptySet()
        }
        return when (val outcome = services.library.searchContents(query = query)) {
            is SearchOutcome.Found -> outcome.documentIds
            SearchOutcome.LibraryNotOpen -> emptySet()
        }
    }

    private data class Inputs(
        val query: String,
        val catalog: CatalogModel?,
        val textMatches: Set<DocumentIdModel>,
        val places: List<PaletteItemModel.GoTo>,
        val busy: Map<OperationLane, String>,
        val aiEnabled: Boolean,
    )

    private companion object {
        const val TYPING_PAUSE_MILLIS = 250L
    }
}
