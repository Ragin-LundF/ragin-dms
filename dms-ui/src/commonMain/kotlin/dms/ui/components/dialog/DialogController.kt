package dms.ui.components.dialog

import dms.ui.components.model.ConfirmationRequestModel
import dms.ui.components.model.TextInputRequestModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * The open question of a view model: a confirmation before a destructive action or a request for a
 * name. The action runs only when the user confirms or submits.
 */
class DialogController {

    private val confirmationState = MutableStateFlow<ConfirmationRequestModel?>(value = null)
    private val inputState = MutableStateFlow<TextInputRequestModel?>(value = null)
    private var onConfirm: (() -> Unit)? = null
    private var onSubmit: ((String) -> Unit)? = null

    val confirmation: StateFlow<ConfirmationRequestModel?> = confirmationState.asStateFlow()
    val input: StateFlow<TextInputRequestModel?> = inputState.asStateFlow()

    fun ask(request: ConfirmationRequestModel, action: () -> Unit) {
        onConfirm = action
        confirmationState.value = request
    }

    fun askText(request: TextInputRequestModel, submit: (String) -> Unit) {
        onSubmit = submit
        inputState.value = request
    }

    fun confirm() {
        val action = onConfirm
        dismissConfirmation()
        action?.invoke()
    }

    fun dismissConfirmation() {
        onConfirm = null
        confirmationState.value = null
    }

    fun submit(text: String) {
        val submit = onSubmit
        dismissInput()
        submit?.invoke(text)
    }

    fun dismissInput() {
        onSubmit = null
        inputState.value = null
    }
}
