package dms.ui.components

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import dms.ui.components.model.TextInputRequest

/** Asks for one line of text; the view model validates what is submitted. */
@Composable
fun TextInputDialog(request: TextInputRequest, onSubmit: (String) -> Unit, onDismiss: () -> Unit) {
    var value by remember(key1 = request) { mutableStateOf(value = request.initialValue) }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(text = request.title) },
        text = {
            OutlinedTextField(
                value = value,
                onValueChange = { value = it },
                label = { Text(text = request.label) },
                singleLine = true,
            )
        },
        confirmButton = {
            TextButton(onClick = { onSubmit(value) }, enabled = value.isNotBlank()) {
                Text(text = request.confirmLabel)
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text(text = "Cancel") }
        },
    )
}
