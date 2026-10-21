package dms.ui.review

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.review.model.ReviewState
import dms.ui.theme.Spacing
import kotlin.math.roundToInt

/** The editable values of an open review, each next to what the document has now. */
@Composable
fun ReviewFields(state: ReviewState, controller: ReviewController) {
    val draft = state.draft
    val current = state.current
    Column(
        modifier = Modifier.heightIn(max = 560.dp).verticalScroll(state = rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        val confidence = state.suggestion.confidence?.let { " · ${(it * 100).roundToInt()} % sure" }.orEmpty()
        Text(
            text = "From ${state.suggestion.model}$confidence. Only the checked values are applied.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        ReviewFieldRow(label = "Title", field = draft.title, current = current.title.value) {
            controller.update(draft = draft.copy(title = it))
        }
        ReviewFieldRow(label = "Date (YYYY-MM-DD)", field = draft.documentDate, current = current.documentDate.value) {
            controller.update(draft = draft.copy(documentDate = it))
        }
        ReviewFieldRow(label = "Correspondent", field = draft.correspondent, current = current.correspondent.value) {
            controller.update(draft = draft.copy(correspondent = it))
        }
        ReviewFieldRow(label = "Category", field = draft.category, current = current.category.value) {
            controller.update(draft = draft.copy(category = it))
        }
        ReviewFieldRow(label = "Tags to add, comma separated", field = draft.tags, current = current.tags.value) {
            controller.update(draft = draft.copy(tags = it))
        }
        ReviewFieldRow(label = "File name", field = draft.fileName, current = current.fileName.value) {
            controller.update(draft = draft.copy(fileName = it))
        }
        state.problem?.let { problem ->
            Text(text = problem, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodyMedium)
        }
    }
}
