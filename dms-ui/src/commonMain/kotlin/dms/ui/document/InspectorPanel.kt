package dms.ui.document

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.domain.document.Document
import dms.domain.taxonomy.TagName
import dms.ui.components.ActionButtons
import dms.ui.components.SectionHeader
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ButtonStyle
import dms.ui.library.model.CategoryEntry
import dms.ui.theme.Spacing

/** Everything about one document, editable: metadata, category, tags, name and place. */
@Composable
fun InspectorPanel(
    document: Document,
    categories: List<CategoryEntry>,
    allTags: List<TagName>,
    busyReason: String?,
    aiBusyReason: String?,
    viewModel: InspectorViewModel,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val enabled = busyReason == null
    Surface(modifier = modifier, color = MaterialTheme.colorScheme.surface) {
        Column(
            modifier = Modifier.verticalScroll(state = rememberScrollState()).padding(all = Spacing.xlarge),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            InspectorHeader(document = document, onClose = onClose)
            DocumentActions(
                document = document,
                busyReason = busyReason,
                aiBusyReason = aiBusyReason,
                viewModel = viewModel,
            )
            DocumentFacts(document = document)
            SectionHeader(title = "Details")
            MetadataEditor(document = document, enabled = enabled) { draft ->
                viewModel.saveMetadata(document = document, draft = draft)
            }
            SectionHeader(title = "Category")
            CategoryPicker(selected = document.categoryId, categories = categories, enabled = enabled) { categoryId ->
                viewModel.assignCategory(document = document, categoryId = categoryId)
            }
            SectionHeader(title = "Tags")
            TagEditor(
                tags = document.tags.sorted(),
                allTags = allTags,
                enabled = enabled,
                onAdd = { viewModel.addTag(document = document, raw = it) },
                onRemove = { viewModel.removeTag(document = document, tag = it) },
            )
            SectionHeader(title = "Remove")
            ActionButtons(
                buttons = listOf(
                    ActionButton(label = "Move to trash…", icon = Icons.Rounded.Delete, style = ButtonStyle.TEXT) {
                        viewModel.trash(document = document)
                    },
                ),
                enabled = enabled,
                busyReason = busyReason,
            )
        }
    }
}
