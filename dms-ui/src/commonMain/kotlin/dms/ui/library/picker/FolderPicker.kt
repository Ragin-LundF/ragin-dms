package dms.ui.library.picker

import dms.domain.library.model.RelativePathModel
import dms.ui.library.picker.model.FolderPickerStateModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

/**
 * The folder picker of "Move to…", for documents and folders alike: a tree of the library's folders to
 * pick one from. It only picks; [open]'s caller moves.
 */
class FolderPicker {

    private val mutableState = MutableStateFlow<FolderPickerStateModel?>(value = null)
    private var onPick: (RelativePathModel) -> Unit = {}

    /** The open picker, or `null` while none is open. */
    val state: StateFlow<FolderPickerStateModel?> = mutableState.asStateFlow()

    /**
     * Opens the picker. [excluded] and everything below it cannot be picked; [onPicked] gets the chosen
     * folder once the user confirms.
     */
    fun open(
        title: String,
        confirmLabel: String = "Move here",
        excluded: RelativePathModel? = null,
        onPicked: (RelativePathModel) -> Unit,
    ) {
        onPick = onPicked
        mutableState.value = FolderPickerStateModel(title = title, confirmLabel = confirmLabel, excluded = excluded)
    }

    fun select(path: RelativePathModel) {
        mutableState.update { state ->
            if (state == null || state.isExcluded(path = path)) state else state.copy(selected = path)
        }
    }

    /** Opens or closes [path] in the tree. */
    fun toggle(path: RelativePathModel) {
        mutableState.update { state ->
            state?.copy(expanded = if (path in state.expanded) state.expanded - path else state.expanded + path)
        }
    }

    /** Hands the selected folder over and closes; nothing happens while none is selected. */
    fun confirm() {
        val selected = mutableState.value?.selected ?: return
        mutableState.value = null
        onPick(selected)
    }

    fun dismiss() {
        mutableState.value = null
    }
}
