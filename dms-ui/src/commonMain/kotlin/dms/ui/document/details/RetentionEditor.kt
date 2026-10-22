package dms.ui.document.details

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import dms.domain.document.model.DocumentModel
import dms.domain.taxonomy.model.CategoryModel
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.theme.Spacing
import java.time.LocalDate

/**
 * The opt-in retention: a keep-until date for this document, or a period for its category. Nothing
 * has one unless the user sets it here; once marked as done, the document is never flagged again.
 */
@Composable
fun RetentionEditor(
    document: DocumentModel,
    category: CategoryModel?,
    dueDate: LocalDate?,
    busyReason: String?,
    onKeepUntil: (String) -> Unit,
    onCategoryYears: (String) -> Unit,
) {
    val savedDate = document.retention?.keepUntil?.toString().orEmpty()
    val savedYears = category?.retentionYears?.toString().orEmpty()
    var keepUntil by remember(key1 = document.id, key2 = savedDate) { mutableStateOf(value = savedDate) }
    var years by remember(key1 = category?.id, key2 = savedYears) { mutableStateOf(value = savedYears) }
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
        val status = when {
            document.retention?.settledAt != null -> "Marked as done; never flagged again"
            dueDate != null -> "Retention ends on $dueDate"
            else -> "No retention set"
        }
        Text(
            text = status,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        OutlinedTextField(
            value = keepUntil,
            onValueChange = { keepUntil = it },
            label = { Text(text = "Keep this document until (YYYY-MM-DD)") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        if (category != null) {
            OutlinedTextField(
                value = years,
                onValueChange = { years = it },
                label = { Text(text = "Keep documents of \"${category.name}\" for … years") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
        }
        ActionButtons(
            buttons = listOfNotNull(
                ActionButtonModel(
                    label = "Save date",
                    disabledReason = "Nothing changed".takeIf { keepUntil == savedDate },
                ) { onKeepUntil(keepUntil) },
                category?.let {
                    ActionButtonModel(
                        label = "Save period",
                        disabledReason = "Nothing changed".takeIf { years == savedYears },
                    ) { onCategoryYears(years) }
                },
            ),
            enabled = busyReason == null,
            busyReason = busyReason,
        )
    }
}
