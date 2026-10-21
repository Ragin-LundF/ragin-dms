package dms.ui.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dms.application.library.model.LibraryCommand
import dms.application.port.ai.model.ModelList
import dms.application.port.state.model.AiSettings
import dms.domain.library.FolderLocation
import dms.domain.library.FolderRole
import dms.domain.library.LibraryFolder
import dms.ui.PlatformActions
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ConfirmationRequest
import dms.ui.review.AiMessages
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.LibraryStatus
import dms.ui.session.model.SessionState
import dms.ui.settings.model.AiConnection
import dms.ui.settings.model.FolderRow
import dms.ui.settings.model.SettingsUiState
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

/**
 * Chooses the primary folder, adds, removes or promotes sync folders, and edits the preferences of
 * this device, including the AI settings and their connection test.
 */
class SettingsViewModel(
    private val services: AppServices,
    private val session: LibrarySession,
    private val platform: PlatformActions,
) : ViewModel() {

    val dialogs: DialogController = DialogController()

    private val connection = MutableStateFlow(value = AiConnection())

    val state: StateFlow<SettingsUiState> = combine(session.state, connection) { sessionState, aiConnection ->
        toUiState(sessionState = sessionState).copy(aiConnection = aiConnection)
    }
        .stateIn(scope = viewModelScope, started = SharingStarted.Eagerly, initialValue = SettingsUiState())

    fun choosePrimaryFolder() {
        viewModelScope.launch {
            val location = platform.chooseFolder(title = "Choose the primary library folder") ?: return@launch
            configure(
                command = LibraryCommand.CreatePrimary(
                    displayName = displayNameOf(location = location),
                    location = FolderLocation(value = location),
                ),
            )
        }
    }

    fun setAddNewFilesWhileScanning(enabled: Boolean) {
        session.setAddNewFilesWhileScanning(enabled = enabled)
    }

    fun detectDates() {
        session.detectDocumentDates()
    }

    fun saveAi(settings: AiSettings) {
        session.updatePreferences { it.copy(ai = settings) }
    }

    /** Asks the server for its models with the typed [settings], without saving them. */
    fun testAi(settings: AiSettings) {
        connection.value = AiConnection(checking = true)
        viewModelScope.launch {
            connection.value = when (val list = services.ai.listAiModels(settings = settings)) {
                is ModelList.Available -> if (list.models.isEmpty()) {
                    AiConnection(
                        message = "The server answers, but has no model loaded. Load one in LM Studio",
                        isError = true,
                    )
                } else {
                    AiConnection(
                        models = list.models,
                        message = "Connected. The server offers ${list.models.size} models",
                    )
                }

                is ModelList.Unavailable -> AiConnection(
                    message = AiMessages.failure(reason = list.reason) + (list.detail?.let { " ($it)" } ?: ""),
                    isError = true,
                )
            }
        }
    }

    fun addSyncFolder() {
        viewModelScope.launch {
            val location = platform.chooseFolder(title = "Choose a sync folder") ?: return@launch
            configure(
                command = LibraryCommand.AddSyncFolder(
                    displayName = displayNameOf(location = location),
                    location = FolderLocation(value = location),
                ),
            )
        }
    }

    fun makePrimary(folder: LibraryFolder) {
        configure(command = LibraryCommand.ChoosePrimary(folderId = folder.id))
    }

    fun removeSyncFolder(folder: LibraryFolder) {
        dialogs.ask(
            request = ConfirmationRequest(
                title = "Remove sync folder",
                message = "The tool stops writing to this folder on this device. Its files stay where they are.",
                affected = listOf(folder.location.value),
                confirmLabel = "Remove",
            ),
        ) {
            configure(command = LibraryCommand.RemoveSyncFolder(folderId = folder.id))
        }
    }

    private fun configure(command: LibraryCommand) {
        session.perform(label = "Updating the library folders") {
            val message = OutcomeMessages.configure(outcome = services.library.configureLibrary(command = command))
            if (message == null) {
                session.reopen()
            } else {
                session.report(message = message)
            }
        }
    }

    private fun toUiState(sessionState: SessionState): SettingsUiState {
        val open = sessionState.status as? LibraryStatus.Open
        val folders = sessionState.folders?.folders.orEmpty()
        return SettingsUiState(
            configured = folders.isNotEmpty(),
            rows = folders.map { folder ->
                val note = when {
                    open == null -> null
                    folder.id in open.foreignFolders -> "Holds another library; left alone"
                    folder.id in open.unreachableFolders -> "Not reachable when the library was opened"
                    else -> null
                }
                FolderRow(folder = folder, isPrimary = folder.role == FolderRole.PRIMARY, note = note)
            },
            busy = sessionState.busy,
            addNewFilesWhileScanning = sessionState.preferences.addNewFilesWhileScanning,
            ai = sessionState.preferences.ai,
        )
    }

    private fun displayNameOf(location: String): String {
        val name = location.trimEnd('/', '\\').substringAfterLast(delimiter = '/').substringAfterLast(delimiter = '\\')
        return name.ifBlank { location }
    }
}
