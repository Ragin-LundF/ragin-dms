package dms.ui.components

import dms.ui.components.model.ConfirmationRequest
import dms.ui.components.model.TextInputRequest
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * The open question of a view model: a confirmation before a destructive action or a request for a
 * name. The action runs only when the user confirms or submits.
 */
class DialogController {

    private val confirmationState = MutableStateFlow<ConfirmationRequest?>(value = null)
    private val inputState = MutableStateFlow<TextInputRequest?>(value = null)
    private var onConfirm: (() -> Unit)? = null
    private var onSubmit: ((String) -> Unit)? = null

    val confirmation: StateFlow<ConfirmationRequest?> = confirmationState.asStateFlow()
    val input: StateFlow<TextInputRequest?> = inputState.asStateFlow()

    fun ask(request: ConfirmationRequest, action: () -> Unit) {
        onConfirm = action
        confirmationState.value = request
    }

    fun askText(request: TextInputRequest, submit: (String) -> Unit) {
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
