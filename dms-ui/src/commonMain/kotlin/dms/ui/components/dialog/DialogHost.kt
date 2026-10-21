package dms.ui.components.dialog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue

/** Shows the open question of [controller], if there is one. */
@Composable
fun DialogHost(controller: DialogController) {
    val confirmation by controller.confirmation.collectAsState()
    val input by controller.input.collectAsState()
    confirmation?.let { request ->
        ConfirmDialog(request = request, onConfirm = controller::confirm, onDismiss = controller::dismissConfirmation)
    }
    input?.let { request ->
        TextInputDialog(request = request, onSubmit = controller::submit, onDismiss = controller::dismissInput)
    }
}
