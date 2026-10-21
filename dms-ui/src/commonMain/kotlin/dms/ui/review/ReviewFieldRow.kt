package dms.ui.review

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.review.model.ReviewField
import dms.ui.theme.Spacing

/**
 * One suggested value: take it or not, edit it, and see what the document has now. With [options] the
 * field also offers them to pick from, and [hint] explains the typed value.
 */
@Composable
fun ReviewFieldRow(
    label: String,
    field: ReviewField,
    current: String,
    options: List<String> = emptyList(),
    hint: String? = null,
    onChange: (ReviewField) -> Unit,
) {
    Row(
        verticalAlignment = Alignment.Top,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        Checkbox(checked = field.accepted, onCheckedChange = { onChange(field.copy(accepted = it)) })
        Column(modifier = Modifier.weight(weight = 1f)) {
            if (options.isEmpty()) {
                OutlinedTextField(
                    value = field.value,
                    onValueChange = { onChange(ReviewField(accepted = true, value = it)) },
                    label = { Text(text = label) },
                    singleLine = true,
                    enabled = true,
                    modifier = Modifier.fillMaxWidth(),
                )
            } else {
                SuggestingTextField(value = field.value, label = label, options = options) {
                    onChange(ReviewField(accepted = true, value = it))
                }
            }
            val now = "Now: ${current.ifEmpty { "—" }}"
            Text(
                text = hint?.let { "$now · $it" } ?: now,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                softWrap = false,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}
