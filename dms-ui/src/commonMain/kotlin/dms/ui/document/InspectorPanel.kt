package dms.ui.document

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.document.Document
import dms.domain.taxonomy.TagName
import dms.ui.components.ActionButtons
import dms.ui.components.FileSizes
import dms.ui.components.SectionHeader
import dms.ui.components.model.ActionButton
import dms.ui.library.model.CategoryEntry
import dms.ui.theme.Spacing

/** Everything about one document, editable: metadata, category, tags, name and place. */
@Composable
fun InspectorPanel(
    document: Document?,
    categories: List<CategoryEntry>,
    allTags: List<TagName>,
    busyReason: String?,
    viewModel: InspectorViewModel,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier.padding(all = Spacing.large).verticalScroll(state = rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        if (document == null) {
            Text(text = "Select a document to see and edit its details.", style = MaterialTheme.typography.bodySmall)
            return@Column
        }
        Text(
            text = document.path.fileName,
            style = MaterialTheme.typography.titleMedium,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            text = "${document.path.parent?.value?.ifEmpty { "Top level" }} · " +
                FileSizes.format(bytes = document.size),
            style = MaterialTheme.typography.bodySmall,
        )
        DocumentActions(document = document, busyReason = busyReason, viewModel = viewModel)
        SectionHeader(title = "Details")
        MetadataEditor(document = document, enabled = busyReason == null) { draft ->
            viewModel.saveMetadata(document = document, draft = draft)
        }
        SectionHeader(title = "Category")
        CategoryPicker(
            selected = document.categoryId,
            categories = categories,
            enabled = busyReason == null,
        ) { categoryId ->
            viewModel.assignCategory(document = document, categoryId = categoryId)
        }
        SectionHeader(title = "Tags")
        TagEditor(
            tags = document.tags.sorted(),
            allTags = allTags,
            enabled = busyReason == null,
            onAdd = { viewModel.addTag(document = document, raw = it) },
            onRemove = { viewModel.removeTag(document = document, tag = it) },
        )
        SectionHeader(title = "Remove")
        ActionButtons(
            buttons = listOf(ActionButton(label = "Move to trash…") { viewModel.trash(document = document) }),
            enabled = busyReason == null,
            busyReason = busyReason,
        )
    }
}
