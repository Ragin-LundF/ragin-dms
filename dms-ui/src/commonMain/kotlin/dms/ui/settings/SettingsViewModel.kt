package dms.ui.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dms.application.library.model.LibraryCommand
import dms.domain.library.FolderLocation
import dms.domain.library.FolderRole
import dms.domain.library.LibraryFolder
import dms.ui.PlatformActions
import dms.ui.components.DialogController
import dms.ui.components.model.ConfirmationRequest
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.LibraryStatus
import dms.ui.session.model.SessionState
import dms.ui.settings.model.FolderRow
import dms.ui.settings.model.SettingsUiState
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

/** Chooses the primary folder and adds, removes or promotes sync folders. */
class SettingsViewModel(
    private val services: AppServices,
    private val session: LibrarySession,
    private val platform: PlatformActions,
) : ViewModel() {

    val dialogs: DialogController = DialogController()

    val state: StateFlow<SettingsUiState> = session.state
        .map { toUiState(sessionState = it) }
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
        )
    }

    private fun displayNameOf(location: String): String {
        val name = location.trimEnd('/', '\\').substringAfterLast(delimiter = '/').substringAfterLast(delimiter = '\\')
        return name.ifBlank { location }
    }
}
