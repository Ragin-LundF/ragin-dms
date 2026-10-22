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
import dms.domain.rule.match.RuleTarget
import dms.ui.review.model.ReviewStateModel
import dms.ui.theme.Spacing
import kotlin.math.roundToInt

/** The values the AI suggested, editable, each next to what the document has now. */
@Composable
fun ReviewFields(state: ReviewStateModel, controller: ReviewController) {
    val draft = state.draft
    val current = state.current
    val suggestion = state.suggestion
    Column(
        modifier = Modifier.heightIn(max = 560.dp).verticalScroll(state = rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        val confidence = suggestion.confidence?.let { " · ${(it * 100).roundToInt()} % sure" }.orEmpty()
        val source = if (suggestion.byRules) "your matching rules, without the AI" else suggestion.model
        Text(
            text = "From $source$confidence. Only the checked values are applied.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        if (suggestion.title != null) {
            ReviewFieldRow(label = "Title", field = draft.title, current = current.title.value) {
                controller.update(draft = draft.copy(title = it))
            }
        }
        if (suggestion.documentDate != null) {
            val now = current.documentDate.value
            ReviewFieldRow(label = "Date (YYYY-MM-DD)", field = draft.documentDate, current = now) {
                controller.update(draft = draft.copy(documentDate = it))
            }
        }
        if (suggestion.correspondent != null) {
            val now = current.correspondent.value
            ReviewFieldRow(label = "Correspondent", field = draft.correspondent, current = now) {
                controller.update(draft = draft.copy(correspondent = it))
            }
        }
        CategoryRow(state = state, controller = controller)
        if (suggestion.tags.isNotEmpty()) {
            ReviewFieldRow(
                label = "Tags to add, comma separated",
                field = draft.tags,
                current = current.tags.value,
                hint = ruleHint(state = state, target = RuleTarget.TAGS),
            ) {
                controller.update(draft = draft.copy(tags = it))
            }
        }
        if (suggestion.fileName != null) {
            ReviewFieldRow(label = "File name", field = draft.fileName, current = current.fileName.value) {
                controller.update(draft = draft.copy(fileName = it))
            }
        }
        TargetFolderRow(state = state, controller = controller)
        FieldRows(state = state, controller = controller)
        state.problem?.let { problem ->
            Text(text = problem, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodyMedium)
        }
    }
}

@Composable
private fun CategoryRow(state: ReviewStateModel, controller: ReviewController) {
    val typed = state.draft.category.value.trim()
    val isNew = ReviewDrafts.isNewCategory(text = typed, paths = state.categoryPaths)
    val created = if (isNew) "\"$typed\" will be created" else null
    ReviewFieldRow(
        label = "Category: pick one or type a new name",
        field = state.draft.category,
        current = state.current.category.value,
        options = state.categoryPaths,
        hint = listOfNotNull(created, ruleHint(state = state, target = RuleTarget.CATEGORY))
            .joinToString(separator = " · ")
            .ifEmpty { null },
    ) {
        controller.update(draft = state.draft.copy(category = it))
    }
}

/** "From rule: <name>" where a matching rule decided the value, so the user sees where it came from. */
private fun ruleHint(state: ReviewStateModel, target: RuleTarget): String? {
    return state.suggestion.ruleNotes[target]?.let { "From rule: $it" }
}

/** "Move to folder": the folder the AI suggests, with its reason, while it suggests one. */
@Composable
private fun TargetFolderRow(state: ReviewStateModel, controller: ReviewController) {
    val target = state.suggestion.targetFolder ?: return
    val reason = state.suggestion.targetReason?.let { "Why: $it" } ?: ruleHint(
        state = state,
        target = RuleTarget.TARGET_FOLDER,
    )
    val created = if (target.isNew) "a new folder" else null
    ReviewFieldRow(
        label = "Move to folder",
        field = state.draft.targetFolder,
        current = state.current.targetFolder.value,
        options = state.folderPaths,
        hint = listOfNotNull(created, reason).joinToString(separator = " · ").ifEmpty { null },
    ) {
        controller.update(draft = state.draft.copy(targetFolder = it))
    }
}

/** One row per custom field the AI filled, labelled like the field. */
@Composable
private fun FieldRows(state: ReviewStateModel, controller: ReviewController) {
    state.draft.fields.forEach { (key, field) ->
        ReviewFieldRow(
            label = state.fieldLabels[key] ?: key.value,
            field = field,
            current = state.current.fields[key]?.value.orEmpty(),
            hint = ruleHint(state = state, target = RuleTarget.FIELDS),
        ) {
            controller.update(draft = state.draft.copy(fields = state.draft.fields + (key to it)))
        }
    }
}
