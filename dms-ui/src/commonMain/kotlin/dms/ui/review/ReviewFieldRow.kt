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

/** One suggested value: take it or not, edit it, and see what the document has now. */
@Composable
fun ReviewFieldRow(label: String, field: ReviewField, current: String, onChange: (ReviewField) -> Unit) {
    Row(
        verticalAlignment = Alignment.Top,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        Checkbox(checked = field.accepted, onCheckedChange = { onChange(field.copy(accepted = it)) })
        Column(modifier = Modifier.weight(weight = 1f)) {
            OutlinedTextField(
                value = field.value,
                onValueChange = { onChange(ReviewField(accepted = true, value = it)) },
                label = { Text(text = label) },
                singleLine = true,
                enabled = true,
                modifier = Modifier.fillMaxWidth(),
            )
            Text(
                text = "Now: ${current.ifEmpty { "—" }}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                softWrap = false,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}
